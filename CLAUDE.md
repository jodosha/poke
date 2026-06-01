# CLAUDE.md

Guidance for AI coding agents working on this repo.

## What this is

`poke` is a small Rust CLI that fires macOS notifications via [`notify-rust`](https://github.com/hoodie/notify-rust). The shipped artifact is a `Poke.app` bundle, not a bare binary — macOS requires a registered bundle id to persist notification permission.

## Project layout

- `src/main.rs` — the entire CLI.
- `Cargo.toml` — deps: `clap` (CLI), `notify-rust` (notifications). No other notification crates.
- `Info.plist` — bundle metadata. `CFBundleIdentifier = com.lucaguidi.poke`.
- `Makefile` — `make app` builds release, assembles `Poke.app`, ad-hoc codesigns it.
- `Poke.app/` — build output, gitignored.

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

2. **Freedesktop path:** uses `Hint::Urgency` for `--severity`, `appname()` for `--app`, and `.action()` + `wait_for_action` for `--target`. None of those APIs exist on `notify-rust`'s macOS backend — they're cfg-gated out. If you need click-to-open on macOS, that's `notify-rust`'s limitation; the macOS arm of `notify-rust`'s own `examples/actions.rs` literally prints "this is a xdg only feature".

## Conventions

- Keep the CLI surface minimal — `clap` derive, one `Args` struct, flags in the order documented in the README.
- Don't bypass `notify-rust` by dropping to `mac-notification-sys` directly. The user explicitly asked for `notify-rust` under the hood.
- Bundle id is `com.lucaguidi.poke`. If it ever changes, update `Info.plist`, the `BUNDLE_ID` var in `Makefile`, and the default in `main.rs` together.
- The macOS `--target` warning lives in `main.rs` — keep it as a `eprintln!` so scripts can suppress it without losing stdout.

## Testing changes

There are no unit tests — this is a thin CLI over `notify-rust`. To verify a change, rebuild and fire a few notifications:

```sh
make app
./Poke.app/Contents/MacOS/Poke --title "test" --message "1" --severity high
./Poke.app/Contents/MacOS/Poke --title "test" --message "2" --timeout 0
./Poke.app/Contents/MacOS/Poke --title "test" --message "3" --target https://example.com  # should print the xdg warning
```

A "Choose Application" picker appearing means `set_application` isn't being called with a valid bundle id — that's a regression.
