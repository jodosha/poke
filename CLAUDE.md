# CLAUDE.md

Guidance for AI coding agents working on this repo.

## What this is

`poke` is a small Rust CLI that fires macOS notifications via [`notify-rust`](https://github.com/hoodie/notify-rust). The shipped artifact is a `Poke.app` bundle, not a bare binary — macOS requires a registered bundle id to persist notification permission.

## Project layout

- `src/main.rs` — the entire CLI.
- `Cargo.toml` — deps: `clap` (CLI), `notify-rust` (notifications). No other notification crates.
- `Info.plist` — bundle metadata. `CFBundleIdentifier = com.lucaguidi.poke`.
- `icon.png` — source-of-truth app icon; converted to `AppIcon.icns` by `make`.
- `Makefile` — `make app` generates the icon, builds release, assembles `Poke.app`, ad-hoc codesigns it.
- `poke` — sh wrapper. Source-of-truth lives at the repo root; `make app` copies it into `Poke.app/Contents/Resources/bin/poke` so it ships inside the bundle. `make install` symlinks `$(BINDIR)/poke` to that path so the CLI is available on `PATH` without the source repo present.
- `Poke.app/`, `AppIcon.icns`, `AppIcon.iconset/` — build output, gitignored.

## Build & run

```sh
make app                                                    # build the bundle
./Poke.app/Contents/MacOS/Poke --title hi --message there   # run
make clean                                                  # remove target/ and Poke.app
```

`cargo build --release` alone works for compilation, but the bare `target/release/poke` binary will not deliver notifications reliably on modern macOS — always test via the `.app` bundle.

## Platform-specific behavior

The codebase has two paths gated on `cfg(target_os = "macos")`:

1. **macOS path:** always calls `notify_rust::set_application(...)` before showing — defaults to `com.lucaguidi.poke`, overridden when `--app` is passed (maps friendly names like `Firefox` → bundle ids in `bundle_id_for`). Skipping this call causes Launch Services to fall back to a `use_default` sentinel and pop a "Choose Application" picker before the notification appears. Do not remove the unconditional `set_application` call.

2. **Freedesktop path:** uses `Hint::Urgency` for `--severity` and `.action()` + `wait_for_action` for `--target`. Neither API exists on `notify-rust`'s macOS backend — they're cfg-gated out. If you need click-to-open on macOS, that's `notify-rust`'s limitation; the macOS arm of `notify-rust`'s own `examples/actions.rs` literally prints "this is a xdg only feature".

### Scheduling (`--in`)

`--in` uses `notify_rust::Notification::schedule_raw(timestamp)`, which **blocks** until the OS fires the banner (mac-notification-sys default). To keep the user's shell free, `main()` detects `--in` and re-spawns the current binary with `POKE_DETACHED=1` in the environment, then exits. The detached child sees the marker, skips the re-spawn branch, and falls through to the blocking `schedule_raw` call. Do not remove the marker check — without it, the child re-spawns infinitely.

Duration parsing is in `parse_duration` and supports compound forms (`1h30m`, `90s`, `2d`). Bare integers (no unit) are rejected on purpose to avoid ambiguity.

## Conventions

- Keep the CLI surface minimal — `clap` derive, one `Args` struct, flags in the order documented in the README.
- Don't bypass `notify-rust` by dropping to `mac-notification-sys` directly. The user explicitly asked for `notify-rust` under the hood.
- Bundle id is `com.lucaguidi.poke`. If it ever changes, update `Info.plist`, the `BUNDLE_ID` var in `Makefile`, and the default in `main.rs` together.
- The macOS `--target` warning lives in `main.rs` — keep it as a `eprintln!` so scripts can suppress it without losing stdout.

## Testing changes

There are no unit tests — this is a thin CLI over `notify-rust`. To verify a change, rebuild and fire a few notifications:

```sh
make app
./poke --title "test" --message "1" --severity high
./poke --title "test" --message "2" --timeout 0
./poke --title "test" --message "3" --target https://example.com   # should print the xdg warning
./poke --title "test" --message "4" --in 10s                       # parent should return instantly; banner fires ~10s later
```

A "Choose Application" picker appearing means `set_application` isn't being called with a valid bundle id — that's a regression.
