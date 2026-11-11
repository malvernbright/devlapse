# devlapse 📸

Captures your desktop every N seconds into timestamped files, one folder per day, so you can turn a developer's day into a timelapse reel.

```
~/Desktop/DevLapse/
└── 2026-11-11/
    ├── 2026-11-11_09-00-00_screen1.jpg
    ├── 2026-11-11_09-00-00_screen2.jpg   ← one file per monitor
    └── 2026-11-11_09-01-00_screen1.jpg
```

## Quick start

```bash
cargo test                        # full suite: core + image encoding
cargo test -- --ignored           # also grabs your real screen once (smoke test)
cargo run --release -- -i 5s -n 3 # 3 captures, 5 s apart: check the output folder
cargo run --release -- -i 30s -a 09:00-19:00   # a real workday
```

Options: `-i/--interval` (`30s`, `5m`, `1h`; default 60s), `-o/--output` (default `~/Desktop/DevLapse`),
`-f/--format` (`jpg` default, or `png`), `-a/--active-hours` (supports overnight like `22:00-02:00`),
`-n/--max-captures`, `-h/--help`.

## Platform notes

- **macOS:** the first run asks for *Screen Recording* permission (System Settings → Privacy & Security).
  Without it, captures show only the wallpaper. Restart the terminal after granting it.
- **Linux:** needs dev packages for xcap, e.g. Ubuntu:
  `sudo apt install libxcb1-dev libxrandr-dev libdbus-1-dev libpipewire-0.3-dev libwayland-dev libegl-dev libgbm-dev`.
  Wayland capture goes through the desktop portal and may prompt you.
- **Windows:** works out of the box. Desktop redirected to OneDrive is detected automatically.

## Design

| Module | Responsibility | Tested with |
|---|---|---|
| `config` | CLI parsing, durations, active hours | pure unit tests |
| `naming` | Sortable, Windows-safe file names | pure unit tests |
| `storage` | Day folders, atomic writes, no overwrites | real temp dirs |
| `session` | One capture → save step; never panics | `FakeCapturer` |
| `scheduler` | Drift-free timer loop, Ctrl+C, skip missed slots | `FakeClock` (hours of time simulated in ms) |
| `xcap_capturer` | Real screen grab + JPEG/PNG encoding | encode tests + `#[ignore]` real capture |

Decisions worth knowing:
- **Captures land on a fixed grid** (09:00:00, 09:01:00…), so a slow capture doesn't make the timestamps creep.
- **After sleep or a stall, missed shots are skipped, not taken in a burst**, which would show up as a stutter in the reel.
- **Writes are atomic** (temp file, then rename), so a crash can't leave a half-written frame that breaks ffmpeg.
- **A failed capture or full disk is logged and the next interval tries again**, so a locked screen doesn't end the day's recording.
- **The core builds without the screenshot library** (`cargo test --no-default-features`), so CI runs headless.

## Making the reel (ffmpeg)

A vertical 1080×1920 reel at 12 fps:

```bash
cd ~/Desktop/DevLapse/2026-11-11
ffmpeg -framerate 12 -pattern_type glob -i '*_screen1.jpg' \
  -vf "scale=1080:1920:force_original_aspect_ratio=decrease,pad=1080:1920:(ow-iw)/2:(oh-ih)/2,format=yuv420p" \
  -c:v libx264 -crf 20 reel.mp4
```

At a 60 s interval, a 10-hour day gives 600 frames, which is a 50-second reel at 12 fps.
Windows ffmpeg has no glob, so use `-f concat` with a file list instead.

**Disk use:** JPEG at 1080p is roughly 200–600 KB per frame, so about 250 MB per screen per 10-hour day at 60 s.

## ⚠️ Before you post

Screenshots capture everything on screen: API keys in `.env` files, Slack DMs, emails, client code, browser tabs.
Scrub every frame before publishing, and consider pausing devlapse (Ctrl+C) during sensitive work.
