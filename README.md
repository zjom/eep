# eep

Keep your computer awake for a while, and watch something nice in the terminal while you wait.

`eep` works like `caffeinate -t` on macOS or `systemd-inhibit sleep` on Linux. It holds a sleep lock for as long as you ask, and plays an animation full-screen in the meantime.

## Install

With Cargo (Rust 1.88 or newer):

```sh
cargo install --path .
```

With Nix:

```sh
nix run github:zjom/eep
```

## Usage

```sh
eep 30m              # stay awake for 30 minutes, random animation
eep 2h -a fire       # pick an animation
eep 1h -n            # no animation, just hold the lock
eep                  # stay awake until you quit
eep -l               # list animations
```

Durations can be written as `90`, `45s`, `20m`, `1h30m`, `2.5h`, `10 min` or `inf`.

| Option                 | Description                                    |
| ---------------------- | ---------------------------------------------- |
| `-a, --animation NAME` | Animation to play (default: `random`)          |
| `-n, --no-animation`   | Hold the lock without an animation             |
| `-l, --list`           | List the animations and exit                   |
| `-d, --display`        | Also try to keep the display awake (see below) |
| `--fps N`              | Frame rate (default: 20)                       |
| `--why TEXT`           | Reason reported to the OS                      |

If output isn't going to a terminal, `eep` skips the animation automatically.

## Keys

| Key                  | Action             |
| -------------------- | ------------------ |
| `n`, space, `→`      | Next animation     |
| `p`, `←`             | Previous animation |
| `f`                  | Toggle fps meter   |
| `q`, `Esc`, `Ctrl-C` | Quit               |

## Animations

| Name     | Description                                          |
| -------- | ---------------------------------------------------- |
| `matrix` | Falling green code rain                              |
| `stars`  | Warp-speed starfield with a slow spin                |
| `fire`   | Fire with flicker and sway                           |
| `plasma` | Colour plasma                                        |
| `life`   | Conway's Game of Life; starts over when it settles   |
| `pipes`  | The old pipes screensaver, drawn with box characters |
| `clock`  | Big countdown in the middle of the screen            |

## How it keeps the machine awake

- **Linux:** `systemd-inhibit --what=sleep --mode=block`. The lock is released when `eep` exits, including if it is killed with SIGKILL.
- **macOS:** `caffeinate -ims -w <pid>`.

`-d` adds `idle` to the Linux lock, or `-d` to `caffeinate` on macOS.

Some Linux desktops ignore the idle lock (GNOME), so the screen may still blank.
