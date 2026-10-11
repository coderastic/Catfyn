//! The app's own settings (`settings.conf` in the config folder): whether the
//! next episode plays by itself, how long its card counts down, and the
//! accent colour (one of the names in `theme::ACCENTS`; no screen for it).

use std::path::{Path, PathBuf};

/// Countdown lengths offered, in seconds.
pub const COUNTDOWNS: [u32; 3] = [5, 10, 15];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Prefs {
    pub autoplay: bool,
    pub countdown: u32,
    pub accent: String,
}

impl Default for Prefs {
    fn default() -> Self {
        Prefs {
            autoplay: true,
            countdown: 10,
            accent: "violet".into(),
        }
    }
}

impl Prefs {
    pub fn parse(text: &str) -> Self {
        let mut p = Prefs::default();
        for line in text.lines() {
            let Some((k, v)) = line.split_once('=') else {
                continue;
            };
            match (k.trim(), v.trim()) {
                ("autoplay", v) => p.autoplay = v == "true",
                ("countdown", v) => {
                    if let Ok(n) = v.parse::<u32>()
                        && COUNTDOWNS.contains(&n)
                    {
                        p.countdown = n;
                    }
                }
                ("accent", v) if crate::theme::ACCENTS.iter().any(|(n, _)| *n == v) => {
                    p.accent = v.to_string()
                }
                _ => {}
            }
        }
        p
    }

    pub fn to_conf(&self) -> String {
        format!(
            "# Catfyn settings\nautoplay = {}\ncountdown = {}\naccent = {}\n",
            self.autoplay, self.countdown, self.accent
        )
    }
}

pub fn path(config: &Path) -> PathBuf {
    config.join("settings.conf")
}

pub fn load(config: &Path) -> Prefs {
    std::fs::read_to_string(path(config))
        .map(|t| Prefs::parse(&t))
        .unwrap_or_default()
}

pub fn save(config: &Path, p: &Prefs) -> std::io::Result<()> {
    let file = path(config);
    if let Some(d) = file.parent() {
        std::fs::create_dir_all(d)?;
    }
    std::fs::write(file, p.to_conf())
}

/// The next countdown length (`delta` = ±1), staying within [`COUNTDOWNS`].
pub fn cycle_countdown(current: u32, delta: i32) -> u32 {
    let i = COUNTDOWNS.iter().position(|&c| c == current).unwrap_or(1) as i32;
    let j = (i + delta).clamp(0, COUNTDOWNS.len() as i32 - 1) as usize;
    COUNTDOWNS[j]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_round_trip_and_countdown() {
        let p = Prefs {
            autoplay: false,
            countdown: 15,
            accent: "teal".into(),
        };
        assert_eq!(Prefs::parse(&p.to_conf()), p);
        assert_eq!(Prefs::parse("countdown = 7"), Prefs::default());
        assert_eq!(cycle_countdown(10, 1), 15);
        assert_eq!(cycle_countdown(15, 1), 15);
        assert_eq!(cycle_countdown(5, -1), 5);
    }
}
