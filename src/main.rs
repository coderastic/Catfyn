//! Catfyn: a Jellyfin client for TV screens.
//!
//! One full-screen window, driven with arrows, OK (Enter) and Back (Esc).
//! The screens' logic lives in `catfyn_core::fin`; this file runs the
//! window, the background jobs (server requests, pictures) and the player.

mod canvas;
mod catfyn;
mod icons;
mod images;
mod jobs;
mod keys;
mod pointer;
mod prefs;
mod target;
mod theme;
mod ui;
mod widgets;

use canvas::Canvas;
use catfyn_core::fin::{Fin, FinEffect, HomeData, Screen};
use catfyn_core::input::{Action, InputMapper};
use catfyn_core::jellyfin::{self as jf, Accounts, Item, Server};
use catfyn_core::scan::Scan;
use eframe::egui;
use jobs::Jobs;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::process::Child;
use std::time::{Duration, Instant};
use target::Target;

enum Msg {
    Servers(Vec<Server>),
    Login(Result<jf::Account, String>),
    Home(String, Result<HomeData, String>),
    Library(String, Result<Vec<Item>, String>),
    Seasons(String, Result<Vec<Item>, String>),
    Episodes(String, Result<Vec<Item>, String>),
    Image(String, Option<String>),
    /// The player has gone; the reporter found no next episode to start
    /// in a fresh player (`Some` only if the old one could not take it).
    PlaybackDone(Option<Box<(jf::Account, Item)>>),
}

struct App {
    fin: Fin,
    accounts: Accounts,
    prefs: prefs::Prefs,
    /// Catfyn's config folder (accounts, settings) and cache (pictures):
    /// `$XDG_CONFIG_HOME/catfyn`, `$XDG_CACHE_HOME/catfyn` (inside a Flatpak,
    /// the app's own folders).
    config: PathBuf,
    cache: PathBuf,
    jobs: Jobs<Msg>,
    mapper: InputMapper,
    pointer: pointer::Pointer,
    hits: Vec<(egui::Rect, Target)>,
    images: images::Images,
    pictures: HashMap<String, Option<String>>,
    wanted: RefCell<Vec<String>>,
    fetching: HashSet<String>,
    toast: Option<(String, Instant)>,
    player: Option<Child>,
    was_focused: bool,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        theme::setup(&cc.egui_ctx);
        let config = catfyn_core::env::config_dir();
        let cache = catfyn_core::env::cache_dir();
        let prefs = prefs::load(&config);
        theme::set_accent(&prefs.accent);
        let accounts = jf::load_accounts(&config);
        let (fin, effects) = Fin::open(&accounts);
        let mut app = App {
            fin,
            accounts,
            prefs,
            config,
            cache,
            jobs: Jobs::new(&cc.egui_ctx),
            mapper: InputMapper::new(),
            pointer: pointer::Pointer::new(),
            hits: Vec::new(),
            images: images::Images::new(),
            pictures: HashMap::new(),
            wanted: RefCell::new(Vec::new()),
            fetching: HashSet::new(),
            toast: None,
            player: None,
            was_focused: true,
        };
        for e in effects {
            app.effect(&cc.egui_ctx, e);
        }
        app
    }

    fn toast(&mut self, ctx: &egui::Context, text: &str) {
        self.toast = Some((text.to_string(), Instant::now() + Duration::from_secs(3)));
        ctx.request_repaint();
    }

    fn save_accounts(&mut self, ctx: &egui::Context) {
        if let Err(e) = jf::save_accounts(&self.config, &self.accounts) {
            self.toast(ctx, &format!("Could not save the accounts ({e})"));
        }
    }

    fn save_prefs(&mut self, ctx: &egui::Context) {
        if let Err(e) = prefs::save(&self.config, &self.prefs) {
            self.toast(ctx, &format!("Could not save the settings ({e})"));
        }
    }

    fn effect(&mut self, ctx: &egui::Context, e: FinEffect) {
        let device = self.accounts.device_id.clone();
        let active = self.accounts.active().cloned();
        match e {
            FinEffect::Discover => self
                .jobs
                .spawn(|| Msg::Servers(jf::discover(Duration::from_millis(2500)))),
            FinEffect::Login {
                server,
                user,
                password,
            } => self
                .jobs
                .spawn(move || Msg::Login(catfyn::login(&device, &server, &user, &password))),
            FinEffect::LoadHome => {
                if let Some(a) = active {
                    let id = a.user_id.clone();
                    self.jobs
                        .spawn(move || Msg::Home(id, catfyn::home(&device, &a)));
                }
            }
            FinEffect::LoadLibrary(lib) => {
                if let Some(a) = active {
                    self.jobs.spawn(move || {
                        let r = catfyn::items(&device, &a, &jf::library_url(&a, &lib));
                        Msg::Library(lib, r)
                    });
                }
            }
            FinEffect::LoadSeasons(series) => {
                if let Some(a) = active {
                    self.jobs.spawn(move || {
                        let r = catfyn::items(&device, &a, &jf::seasons_url(&a, &series));
                        Msg::Seasons(series, r)
                    });
                }
            }
            FinEffect::LoadEpisodes { series, season } => {
                if let Some(a) = active {
                    self.jobs.spawn(move || {
                        let r = catfyn::items(&device, &a, &jf::episodes_url(&a, &series, &season));
                        Msg::Episodes(season, r)
                    });
                }
            }
            FinEffect::Play { item, from_start } => {
                if let Some(a) = active {
                    self.play(ctx, a, item, from_start);
                }
            }
            FinEffect::SwitchAccount(i) => {
                if i < self.accounts.list.len() {
                    self.accounts.active = i;
                    self.save_accounts(ctx);
                }
            }
            FinEffect::SetIncognito(i, on) => {
                if let Some(a) = self.accounts.list.get_mut(i) {
                    a.incognito = on;
                    self.save_accounts(ctx);
                    self.toast(
                        ctx,
                        if on {
                            "Incognito: nothing is marked as watched"
                        } else {
                            "Watch progress is saved again"
                        },
                    );
                }
            }
            FinEffect::SignOut(i) => {
                self.accounts.remove(i);
                self.save_accounts(ctx);
            }
            FinEffect::Toast(t) => self.toast(ctx, &t),
            FinEffect::ToggleAutoplay => {
                self.prefs.autoplay = !self.prefs.autoplay;
                self.save_prefs(ctx);
            }
            FinEffect::CountdownStep(d) => {
                let secs = prefs::cycle_countdown(self.prefs.countdown, d as i32);
                if secs != self.prefs.countdown {
                    self.prefs.countdown = secs;
                    self.save_prefs(ctx);
                }
            }
            FinEffect::Close => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
        }
    }

    fn play(&mut self, ctx: &egui::Context, a: jf::Account, item: Item, from_start: bool) {
        if let Some(mut old) = self.player.take() {
            let _ = old.kill();
            let _ = old.wait();
        }
        let device = self.accounts.device_id.clone();
        match catfyn::start_player(&device, &a, &item, from_start) {
            Ok(child) => {
                self.player = Some(child);
                let (tx, rctx) = self.jobs.sender();
                let autoplay = catfyn::Autoplay {
                    enabled: self.prefs.autoplay,
                    countdown: self.prefs.countdown,
                };
                catfyn::spawn_reporter(
                    device,
                    a.clone(),
                    item,
                    self.cache.clone(),
                    autoplay,
                    move |next| {
                        let _ = tx.send(Msg::PlaybackDone(next.map(|n| Box::new((a, n)))));
                        rctx.request_repaint();
                    },
                );
            }
            Err(e) => self.toast(ctx, &e),
        }
    }

    fn message(&mut self, ctx: &egui::Context, msg: Msg) {
        match msg {
            Msg::Servers(v) => self.fin.on_discovered(v),
            Msg::Login(r) => {
                let ok = match r {
                    Ok(a) => {
                        self.accounts.add(a);
                        self.save_accounts(ctx);
                        Ok(())
                    }
                    Err(e) => Err(e),
                };
                for e in self.fin.on_login(ok) {
                    self.effect(ctx, e);
                }
            }
            Msg::Home(user, r) => {
                if self.accounts.active().is_some_and(|a| a.user_id == user) {
                    self.fin.on_home(r);
                }
            }
            Msg::Library(id, r) => self.fin.on_library(&id, r),
            Msg::Seasons(id, r) => {
                for e in self.fin.on_seasons(&id, r) {
                    self.effect(ctx, e);
                }
            }
            Msg::Episodes(season, r) => self.fin.on_episodes(&season, r),
            Msg::Image(url, path) => {
                self.fetching.remove(&url);
                self.pictures.insert(url, path);
            }
            Msg::PlaybackDone(next) => {
                if let Some(mut p) = self.player.take() {
                    let _ = p.wait();
                }
                if let Some(b) = next {
                    let (a, item) = *b;
                    self.play(ctx, a, item, false);
                } else {
                    // Fresh "Continue watching" after the film.
                    ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
                    if matches!(self.fin.screen(), Some(Screen::Browse)) {
                        self.effect(ctx, FinEffect::LoadHome);
                    }
                }
            }
        }
    }

    /// Download the pictures the last frame asked for, a few at a time.
    fn fetch_pictures(&mut self) {
        let wanted = std::mem::take(&mut *self.wanted.borrow_mut());
        for url in wanted {
            if self.pictures.contains_key(&url)
                || self.fetching.contains(&url)
                || self.fetching.len() >= 6
            {
                continue;
            }
            self.fetching.insert(url.clone());
            let cache = self.cache.clone();
            self.jobs.spawn(move || {
                let path = catfyn::fetch_image(&cache, &url);
                Msg::Image(url, path)
            });
        }
    }

    fn loading(&self) -> bool {
        matches!(self.fin.home, Scan::Busy)
            || matches!(
                self.fin.screen(),
                Some(
                    Screen::Connect {
                        servers: Scan::Busy,
                        ..
                    } | Screen::SignIn { busy: true, .. }
                        | Screen::Library {
                            items: Scan::Busy,
                            ..
                        }
                )
            )
    }
}

fn hit_test(hits: &[(egui::Rect, Target)], pos: egui::Pos2) -> Option<Target> {
    hits.iter()
        .rev()
        .find(|(r, _)| r.contains(pos))
        .map(|(_, t)| *t)
}

impl eframe::App for App {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        for msg in self.jobs.drain() {
            self.message(ctx, msg);
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        let focused = keys::window_focused(&ctx);
        if !focused && self.was_focused {
            self.mapper.reset();
        }
        self.was_focused = focused;

        let events = keys::collect(&ctx);
        let actions = self
            .mapper
            .map(&events, keys::now(&ctx), self.fin.text_mode());
        let p = self.pointer.collect(&ctx);
        let mut effects = Vec::new();
        for action in actions {
            effects.extend(self.fin.handle(action, &self.accounts));
        }
        // Mouse: click = focus + OK, right click = Back, wheel = Up/Down.
        // Hit rects are from the previous frame.
        if let Some(pos) = p.click {
            match hit_test(&self.hits, pos) {
                Some(Target::Fin(r, c)) => {
                    if self.fin.focus(r, c, &self.accounts) {
                        effects.extend(self.fin.handle(Action::Ok, &self.accounts));
                    }
                }
                Some(Target::Outside) | None => {}
            }
        }
        if p.back {
            effects.extend(self.fin.handle(Action::Back, &self.accounts));
        }
        let (n, action) = if p.wheel > 0 {
            (p.wheel, Action::Up)
        } else {
            (-p.wheel, Action::Down)
        };
        for _ in 0..n.min(5) {
            effects.extend(self.fin.handle(action, &self.accounts));
        }
        for e in effects {
            self.effect(&ctx, e);
        }
        if self.mapper.needs_tick() {
            ctx.request_repaint();
        }

        let c: Canvas<Target> = Canvas::new(ui, 0);
        let now = keys::now(&ctx);
        c.background_at(None, false, now);
        let pics = ui::fin::FinPics {
            images: &self.images,
            cache: &self.pictures,
            wanted: &self.wanted,
        };
        ui::fin::draw(&c, &self.fin, &self.accounts, &self.prefs, &pics, now);
        let at = Instant::now();
        if matches!(&self.toast, Some((_, until)) if at >= *until) {
            self.toast = None;
        }
        if let Some((text, until)) = &self.toast {
            let left = until.saturating_duration_since(at).as_secs_f32();
            widgets::toast(&c, text, (left / 0.3).clamp(0.0, 1.0));
        }
        self.hits = c.take_hits();
        self.fetch_pictures();
        if self.toast.is_some() || self.loading() || !self.fetching.is_empty() {
            ctx.request_repaint_after(Duration::from_millis(16));
        }
    }
}

impl Drop for App {
    fn drop(&mut self) {
        if let Some(mut p) = self.player.take() {
            let _ = p.kill();
            let _ = p.wait();
        }
    }
}

/// `CATFYN_RENDERER=glow|wgpu` forces a renderer; by default wgpu, and if
/// that cannot start (no Vulkan) Catfyn starts again with OpenGL.
fn renderer() -> (eframe::Renderer, bool) {
    match std::env::var("CATFYN_RENDERER").ok().as_deref() {
        Some("glow") => (eframe::Renderer::Glow, true),
        Some("wgpu") => (eframe::Renderer::Wgpu, true),
        _ => (eframe::Renderer::Wgpu, false),
    }
}

fn main() -> eframe::Result {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--version" || a == "-V") {
        println!("catfyn {}", catfyn_core::env::version());
        return Ok(());
    }
    if args.iter().any(|a| a == "--help" || a == "-h") {
        println!(
            "catfyn {}: a Jellyfin client for TV screens\n\n\
             Usage: catfyn [--windowed]\n\n\
             Keys: arrows move, Enter = OK (hold for options), Esc/Backspace = Back.\n\
             Settings and accounts live in ~/.config/catfyn/ (Flatpak: ~/.var/app/io.github.coderastic.Catfyn/config/catfyn/).",
            catfyn_core::env::version()
        );
        return Ok(());
    }
    let windowed = args.iter().any(|a| a == "--windowed")
        || std::env::var("CATFYN_WINDOWED").is_ok_and(|v| v == "1");
    let (renderer, explicit) = renderer();
    let viewport = egui::ViewportBuilder::default()
        .with_title("Catfyn")
        .with_app_id("io.github.coderastic.Catfyn")
        .with_decorations(windowed)
        .with_fullscreen(!windowed)
        .with_inner_size([1280.0, 720.0]);
    let options = eframe::NativeOptions {
        viewport,
        renderer,
        ..Default::default()
    };
    let result = eframe::run_native("Catfyn", options, Box::new(|cc| Ok(Box::new(App::new(cc)))));
    if let Err(e) = &result
        && !explicit
    {
        eprintln!("catfyn: wgpu renderer failed ({e}); retrying with OpenGL");
        if let Ok(exe) = std::env::current_exe() {
            use std::os::unix::process::CommandExt;
            let err = std::process::Command::new(exe)
                .args(std::env::args_os().skip(1))
                .env("CATFYN_RENDERER", "glow")
                .exec();
            eprintln!("catfyn: could not restart: {err}");
        }
    }
    result
}
