# {{crate_name}}

A PocketBook e-reader application built on
{% if framework == "slint" %}[inkview-slint](https://github.com/simmsb/inkview-rs), which runs a
[Slint](https://slint.dev/) UI on the reader's e-ink screen.
{% elsif framework == "embedded-graphics" %}[inkview-eg](https://github.com/simmsb/inkview-rs), which
exposes the reader's screen as an [embedded-graphics](https://docs.rs/embedded-graphics) `DrawTarget`.
{% else %}[inkview](https://github.com/simmsb/inkview-rs), calling the PocketBook SDK directly through
its FFI bindings.
{% endif %}

## What's in the box

{% if framework == "slint" %}
The demo is a circle drawer with undo/redo: tapping the background adds a circle, tapping a circle
selects it for resizing.

- `ui/main.slint` — the `MainWindow` component; the UI itself lives here.
- `src/bin/{{crate_name}}.rs` — wires Slint callbacks to the model and drives the undo stack.
- `build.rs` — compiles `ui/main.slint` at build time and embeds glyphs for the software renderer.
{% elsif framework == "embedded-graphics" %}
The demo draws a border, a triangle, a filled square, a circle, and centered text.

- `src/bin/{{crate_name}}.rs` — the `draw_content` function is where the drawing happens; the
  surrounding `main` handles the inkview event loop and flushes the display on `Show`/`Repaint`.
{% else %}
The demo clears the screen, draws two nested grey rectangles and a line, and renders "Hello world!"
centered in `LiberationSans`.

- `src/bin/{{crate_name}}.rs` — the `Event::Init` arm is where the drawing happens; `FullUpdate`
  copies the buffer to the physical screen.
{% endif %}
- `justfile` — the whole build/deploy interface. Run `just --list` to see every recipe.
- `src/lib.rs` — empty on purpose, so the crate has a lib+bin layout you can grow into.

## Prerequisites

Like [inkview-rs](https://github.com/simmsb/inkview-rs) itself, this project depends on:

1. [Zig](https://ziglang.org/learn/getting-started/#installing-zig) — required by `cargo-zigbuild`,
   which performs the cross-linking against the reader's old glibc.
1. [just](https://github.com/casey/just) — provides the pre-configured helper commands.
1. [Nix](https://nixos.org/download/) + [direnv](https://direnv.net/docs/installation.html) +
   [devenv](https://devenv.sh/getting-started/) — Nix supplies a sandbox with the exact
   cross-compilation toolchain the e-reader needs, and direnv injects it into your shell the moment
   you enter the project directory.

Once those are installed, allow the environment:

```bash
direnv allow
```

The first entry into the shell downloads and builds the cross toolchain and takes a while;
subsequent ones are instant.

## Building

```bash
just build                        # dev profile
just cargo_profile=release build  # release profile
```

`cargo build` on its own will **not** produce a binary that runs on the device. The recipe uses
`cargo zigbuild --target armv7-unknown-linux-gnueabi.2.23`, and that `.2.23` glibc suffix is what
makes the result load on the reader's userspace.
{% if framework == "slint" %}
On macOS, debug builds of the Slint variant exhaust the default open-file limit. The `build` recipe
raises it to 4096 for the duration of the build, so no manual `ulimit` step is needed.
{% endif %}

## Configuration

Every knob is a variable at the top of the `justfile`, overridden on the command line:

```bash
just cargo_profile=release pb_device=PB632 deploy-usb
```

| Variable | Default | Purpose |
| --- | --- | --- |
| `cargo_profile` | `dev` | Cargo build profile. |
| `pb_sdk_version` | `6.10` | PocketBook SDK — selects the `sdk-6-10` Cargo feature. Also `5.19`, `6.5`, `6.8`. Exactly one is active per build. |
| `pb_device` | `PB632` | Volume name of the device when mounted over USB. |
| `pb_ssh_user` | `reader` | SSH username on the device. |
| `pb_ssh_ip` | `192.168.1.27` | **Placeholder.** The device is on DHCP — override per invocation. |
| `pb_ssh_port` | `2222` | SSH port on the device. |
| `pb_target_app_dir` | `/mnt/ext1/applications` | Where the app lands on the device. |
| `pb_target_app_name` | `{{crate_name}}.app` | Filename the binary is deployed as. |
| `pb_emu_sdk_version` | `6.10` | SDK the emulator uses. Independent of `pb_sdk_version`; `6.8` also works. |
| `pb_emu_device` | `632` | Device identity the emulator reports. |
| `pb_emu_resmode` | `8` | Emulated screen geometry — `8` is 1072x1448. The justfile lists the other values. |

## Running on your computer (emulator)

You do not need the device to see the app. The PocketBook SDK ships a **host x86_64 build of
`libinkview.so`**, and `inkview` loads its library at runtime rather than linking it, so the same
code runs on a desktop and draws into an X11 window.

The tooling lives in [inkview-rs-emu](https://github.com/ihrfv/inkview-rs-emu), shared by every inkview-rs
project rather than vendored into this one, so it can be updated without regenerating. Install it
once per machine:

```bash
git clone https://github.com/ihrfv/inkview-rs-emu ~/tools/inkview-rs-emu
export PATH="$HOME/tools/inkview-rs-emu:$PATH"   # add to your shell profile
```

It needs Docker **running** (Docker Desktop does not start itself) and, on macOS,
[XQuartz](https://www.xquartz.org) — `run-emu` configures that for you:

```bash
brew install --cask xquartz             # macOS only

just fetch-emu-assets                   # one-time, 0.6-1.3 GB download, ~40 MB kept
just emu-image                          # one-time, builds the container
just pb_emu_resmode=3 build-run-emu     # build for the host, open a window
just build-screenshot-emu               # headless -> target/emu/screenshot.png
```

Assets are cached in `~/.cache/inkview-emu` and shared across projects, so the download happens once
per machine, not once per project.

To *drive* the app — tap, swipe, keys — add inkview-rs-emu's `inkview-pilot` crate; its README has the
few lines of wiring. The emulator's window drops synthetic X11 clicks, so that crate is the only way
to interact with it programmatically.

### Caveats

- `pb_emu_resmode` sets the window size and X11 does not scale it, so the 1072x1448 default
  overflows most laptop screens. Pass `pb_emu_resmode=3` (828x1200) for interactive runs and leave
  it at the default for screenshots, where the real geometry is what you want.
- `pb_emu_sdk_version` accepts `6.10` (default) and `6.8`; other SDK archives ship no host build.
  The two use different containers — see inkview-rs-emu's README.
- The emulator approximates the panel but not e-ink refresh behaviour, so timing-sensitive update
  code still has to be checked on real hardware. For firmware-level questions, inkview-emu documents
  escalating to pbemu.
- On Apple Silicon, enable *Use Rosetta for x86/amd64 emulation* in Docker Desktop to keep it fast.
- `just emu-shell` drops into the container with `ldd`, `readelf` and `xdpyinfo` when a binary
  refuses to start.

## Deploying

See [inkview-rs](https://github.com/simmsb/inkview-rs) for deployment strategies beyond the two
below.

### Over USB

Connect the reader, then:

```bash
just cargo_profile=release build-deploy-usb
```

This builds, copies the binary onto the mounted volume, cleans up macOS metadata files, and flushes
the filesystem cache. `pb_device` must match the volume name — known values are `PB632`
(Touch HD 3), `PB626` (Touch Lux 3), and `6678-3C5A` (InkPad 4). Use `deploy-usb` on its own to skip
the build.

### Over SSH

Load your SSH key once per shell — this has to run in your own shell, not in a recipe, since a
`just` recipe cannot export into the shell that invoked it:

```bash
eval "$(ssh-agent -s)" && ssh-add ~/.ssh/id_rsa
```

Then build and deploy in one step:

```bash
just cargo_profile=release pb_ssh_ip="192.168.1.42" build-deploy-ssh
```

The binary is copied to a `.stage` path first, then atomically renamed over the live app, which is
killed and relaunched — so an interrupted transfer leaves the previous version intact.

**Note:** large transfers to the device fail intermittently, leaving no partial file. Just retry. To
be certain the copy is good, compare `md5sum` on the device against `md5` locally.

## Running on the device

The app does not appear on screen immediately. It starts in the background — hold the home button
for about 3 seconds and pick it from the list of open applications.
