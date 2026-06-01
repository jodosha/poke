# poke

Tiny Rust CLI that delivers macOS notifications via [`notify-rust`](https://github.com/hoodie/notify-rust).

## Build

```sh
make app
```

Produces `Poke.app` — an ad-hoc codesigned bundle with id `com.lucaguidi.poke`. The bundle is required so macOS can persist notification permission across runs; without it the system treats every invocation as a new requester and re-prompts.

The first run will trigger a permission prompt. Approve it once, and consent sticks.

## Usage

The repo ships a `poke` shell wrapper next to `Poke.app` — use that for everyday invocation:

```sh
./poke \
  --title "Build done" \
  --message "All green ✓" \
  --timeout 5 \
  --severity high
```

Symlink it onto your `PATH` to call `poke` from anywhere:

```sh
ln -s "$PWD/poke" ~/.local/bin/poke
poke --title hi --message there
```

You can also invoke the bundled binary directly, or via `open`:

```sh
./Poke.app/Contents/MacOS/Poke --title hi --message there
open Poke.app --args --title hi --message there
```

### Flags

| Flag         | Description                                                              |
| ------------ | ------------------------------------------------------------------------ |
| `--title`    | Notification title. Required.                                            |
| `--message`  | Notification body. Required.                                             |
| `--timeout`  | Seconds before the banner dismisses. `0` = sticky. Default `5`.          |
| `--severity` | `low`, `normal`, or `high`. Default `normal`.                            |
| `--target`   | Path or URL opened on click. **Freedesktop-only**; ignored on macOS.     |

### macOS notes

- `--severity` maps to `Hint::Urgency` on Linux. macOS notifications don't carry urgency hints, so the flag is accepted but has no visible effect on the banner.
- `--target` requires the freedesktop notification spec (action buttons + `wait_for_action`), which `notify-rust` doesn't implement on macOS. On macOS the flag prints a warning and is ignored.

## Clean

```sh
make clean
```

Removes `target/` and `Poke.app`.
