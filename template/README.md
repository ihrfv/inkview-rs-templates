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
**On macOS**, debug builds of the Slint variant fail unless the open-file limit is raised:

```bash
ulimit -n 4096
```

`just preconfigure-build-and-deploy-ssh` does this for you.
{% endif %}

## Configuration

Every knob is a variable at the top of the `justfile`, overridden on the command line:

```bash
just cargo_profile=release pb_device=PB632 deploy-usb
```

| Variable | Default | Purpose |
| --- | --- | --- |
| `cargo_profile` | `dev` | Cargo build profile. |
| `pb_sdk_version` | `6.10` | PocketBook SDK — selects the `sdk-6-10` Cargo feature. Also `5.19`, `6.5`, `6.8`. |
| `pb_device` | `PB632` | Volume name of the device when mounted over USB. |
| `pb_ssh_user` | `reader` | SSH username on the device. |
| `pb_ssh_ip` | `192.168.1.27` | **Placeholder.** The device is on DHCP — override per invocation. |
| `pb_ssh_port` | `2222` | SSH port on the device. |
| `pb_target_app_dir` | `/mnt/ext1/applications` | Where the app lands on the device. |
| `pb_target_app_name` | `{{crate_name}}.app` | Filename the binary is deployed as. |

## Deploying

See [inkview-rs](https://github.com/simmsb/inkview-rs) for deployment strategies beyond the two
below.

### Over USB

Connect the reader, then:

```bash
just cargo_profile=release deploy-usb
```

This copies the binary onto the mounted volume, cleans up macOS metadata files, and flushes the
filesystem cache. `pb_device` must match the volume name — known values are `PB632`
(Touch HD 3), `PB626` (Touch Lux 3), and `6678-3C5A` (InkPad 4).

### Over SSH

Once per shell, raise the file-descriptor limit and load your SSH key:

```bash
just preconfigure-build-and-deploy-ssh
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
