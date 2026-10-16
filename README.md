# Catfyn

**A no-nonsense Jellyfin client for TV interfaces.**

I wanted to watch my own Jellyfin library on the TV without a browser tab,
a mouse, or an app that tries to be a whole media center. Catfyn is the
result: one full-screen window you drive from the couch with arrows, OK
and Back. Pick something, it plays. That's it.

## What it does

- It finds your server on the network by itself, or you type the address.
- Several accounts, on one or more servers, and you switch between them in
  Settings. An account can be incognito, so nothing you watch on it gets
  marked as watched.
- The home screen has Continue watching, Next up and the newest things in
  each library. Your libraries are tabs at the top.
- Series work the way you'd expect: seasons, and episodes showing what you
  watched and how much is left.
- Videos play in mpv, full screen, with controls made for a remote. OK
  shows them, OK again pauses, Left and Right skip 10 seconds. Subtitles
  and audio tracks are a button away, and Catfyn follows the subtitle
  settings of your Jellyfin user, external subtitle files included.
- When the end credits start, a card counts down to the next episode (Play
  now or Cancel). After three episodes in a row it asks if you're still
  watching. You can turn this off or change the countdown in Settings.
- It reports your progress to the server, so you can stop on the TV and
  carry on anywhere else.

What it doesn't do: music, live TV, downloads, transcoding settings,
plugins. It plays video, and tries to do that well.

## Install

Grab one of these from the [Releases](../../releases) page.

### Flatpak (recommended)

Everything is inside, mpv included, and it works on any distribution.

```bash
flatpak install --user Catfyn-1.0.0-x86_64.flatpak
flatpak run io.github.coderastic.Catfyn
```

It needs the Freedesktop 25.08 runtime; if you have Flathub set up,
Flatpak fetches it for you.

### AppImage

Also comes with its own mpv, and runs on distributions from about 2024 on
(glibc 2.39 or newer, so Ubuntu 24.04, Debian 13, Fedora 40 and later).
Make it executable and run it:

```bash
chmod +x Catfyn-1.0.0-x86_64.AppImage
./Catfyn-1.0.0-x86_64.AppImage
```

### Tarball

The smallest option, for people who'd rather use their distribution's mpv.
You need `mpv` and `curl` installed.

```bash
tar xf catfyn-1.0.0-x86_64.tar.gz
cd catfyn-1.0.0-x86_64
./install.sh                 # into ~/.local, or: sudo ./install.sh /usr/local
```

`./install.sh --uninstall` removes it again.

## Using it

| Key | Does |
|---|---|
| Arrow keys | Move |
| Enter | OK. Hold it for options (e.g. *Play from start*) |
| Esc or Backspace | Back. Back on the home screen closes Catfyn |
| Typing | Goes straight into text fields (server address, user name, password) |

The mouse works too: click is OK, right click is Back, the wheel scrolls.

Most TV remotes that come with a USB dongle, and most Bluetooth remotes,
show up as a keyboard, so they just work. Game controllers aren't
supported directly; a tool like AntiMicroX can map one to the keys above.

**In the player**: OK shows the controls (Pause, Subtitles, Audio, …),
OK again pauses. Left/Right skip 10 s. With the controls open, Up gets you
to the seek bar, where holding Left/Right skips a minute at a time. When a
series episode plays there's also a Previous episode button. Back
hides the controls, or stops when they're hidden. Media keys on a keyboard or
remote work as you'd expect.

**Settings** are behind your account button at the top right: accounts
(switch, add, incognito, sign out), whether the next episode plays by
itself, and how long the countdown is.

Catfyn starts full screen. `catfyn --windowed` gives you a normal window,
which is handy on a desktop.

### Where things are kept

- `~/.config/catfyn/accounts.conf`: your accounts and their sign-in
  tokens (readable only by you). Your password is never stored.
- `~/.config/catfyn/settings.conf`: autoplay, countdown, and `accent`, the
  highlight colour (violet, periwinkle, blue, teal, green, amber, rose,
  red or silver).
- `~/.cache/catfyn/`: posters and pictures. Safe to delete.

The Flatpak keeps the same files under
`~/.var/app/io.github.coderastic.Catfyn/` instead.

## Building

You need Rust (1.88 or newer) and, to run it, mpv and curl.

```bash
cargo build --release
./target/release/catfyn --windowed
```

The packages are built with the scripts in `packaging/`. `stage.sh` runs
on Debian 13 (a chroot or container is fine): it builds Catfyn and
collects mpv with the libraries it needs, so the packages run on any
distribution with glibc 2.39 or newer. The rest can run anywhere:

```bash
./packaging/stage.sh         # on Debian 13: dist/stage
./packaging/tarball.sh       # dist/catfyn-<version>-x86_64.tar.gz
./packaging/appimage.sh      # dist/Catfyn-<version>-x86_64.AppImage (needs appimagetool)
./packaging/flatpak.sh       # dist/Catfyn-<version>-x86_64.flatpak (needs the 25.08 runtime)
```

## License

MIT, see [LICENSE](LICENSE). Catfyn includes the
[Inter](https://rsms.me/inter/) typeface, under the SIL Open Font License
(`assets/fonts/LICENSE-Inter.txt`). Jellyfin is a trademark of its
respective owners; Catfyn is an independent client.
