# poke

Tiny Rust CLI that delivers macOS notifications via [`notify-rust`](https://github.com/hoodie/notify-rust).

## Build

```sh
make app
```

Produces `Poke.app` — an ad-hoc codesigned bundle with id `com.lucaguidi.poke`. The bundle is required so macOS can persist notification permission across runs; without it the system treats every invocation as a new requester and re-prompts.

The first run triggers a permission prompt. Approve it once, and consent sticks.

## Install

```sh
make install                       # PREFIX=/Applications, BINDIR=/usr/local/bin
PREFIX=~/Applications make install # user-level install
sudo make install                  # if BINDIR isn't writable
```

`make install` copies `Poke.app` to `$(PREFIX)` and symlinks `$(BINDIR)/poke` to the wrapper script that ships **inside** the bundle at `Contents/Resources/bin/poke`. That means:

- The wrapper travels with the app — copy `Poke.app` to any other Mac and a single `ln -s` brings the CLI back.
- Uninstalling the app (`make uninstall`, or just `rm -rf` the bundle) leaves a dangling `poke` symlink that's easy to remove.

After install, `poke` is on your `PATH`:

```sh
poke --title "Build done" --message "All green ✓" --severity high
```

### Distributing without this repo

If you copy `Poke.app` to a machine that doesn't have the source:

```sh
cp -R Poke.app /Applications/
ln -s /Applications/Poke.app/Contents/Resources/bin/poke /usr/local/bin/poke
```

That's it — no Rust toolchain needed on the target machine.

## Run without installing

```sh
./poke --title hi --message there
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
| `--in`       | Schedule delivery for later. Compound: `30m`, `1h`, `90s`, `1h30m`, `2d`. macOS only. |

### Scheduling

`--in` schedules the notification for later and returns immediately:

```sh
poke --title "Wind down" --message "Code freeze in 30 minutes" --severity low --in 30m
```

`notify-rust`'s scheduled-delivery API blocks the calling process until the OS fires the banner, so `poke` re-spawns itself as a detached background process when `--in` is set. The parent exits within milliseconds; the child sleeps until the delivery time and then exits silently. No new shell or `&` needed.

### macOS notes

- `--severity` maps to `Hint::Urgency` on Linux. macOS notifications don't carry urgency hints, so the flag is accepted but has no visible effect on the banner.
- `--target` requires the freedesktop notification spec (action buttons + `wait_for_action`), which `notify-rust` doesn't implement on macOS. On macOS the flag prints a warning and is ignored.

## Clean

```sh
make clean
```

Removes `target/` and `Poke.app`.
