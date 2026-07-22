# Overview

This repository is adjacent to [inkview-rs](https://github.com/simmsb/inkview-rs/tree/master).
It provides you with the templating mechanism to create new configured vanilla `inkview`, `inkview-eg`, and `inkview-slint` projects.

## Prerequisites

To generate a project you need `cargo-generate`:
```bash
cargo install cargo-generate
```

The *generated* project cross-compiles for the reader and needs
[Nix](https://nixos.org/download/) + [direnv](https://direnv.net/docs/installation.html) +
[devenv](https://devenv.sh/getting-started/), plus [Zig](https://ziglang.org/learn/getting-started/#installing-zig)
and [just](https://github.com/casey/just). The template ships a `devenv.nix`/`.envrc` that supplies
the cross toolchain, but `just build` will not work until direnv has loaded it — see the generated
project's own README.

## Creating a new project

Here is the syntax to create a new project from this template:
```bash
cargo generate --git https://github.com/ihrfv/inkview-rs-templates.git template --name <PROJECT_NAME> --define framework=<FRAMEWORK_TYPE>
```

`<FRAMEWORK_TYPE>` can be:
* `none` - for the vanilla `inkview` example project
* `slint` - for `inkview-slint` example project
* `embedded-graphics` - for `inkview-eg` example project

For example:
```bash
cargo generate --git https://github.com/ihrfv/inkview-rs-templates.git template --name test-slint --define framework=slint
```

To generate from a local checkout of this repository instead (this is what CI does):
```bash
cargo generate --path . template --name test-slint --define framework=slint --destination /tmp
```

To test that it works, execute inside of the newly generated project:
```bash
direnv allow   # first run builds the cross toolchain and takes a while
just build
```

Generated projects can also run on your computer, without the reader, using the emulator that ships
inside the PocketBook SDK. That is driven by [inkview-rs-emu](https://github.com/ihrfv/inkview-rs-emu),
installed once per machine rather than vendored into each project; it needs Docker (and XQuartz on
macOS):
```bash
git clone https://github.com/ihrfv/inkview-rs-emu ~/tools/inkview-rs-emu
export PATH="$HOME/tools/inkview-rs-emu:$PATH"

just fetch-emu-assets                 # one-time, 0.6-1.3 GB download
just emu-image
just pb_emu_resmode=3 build-run-emu   # or `just build-screenshot-emu` when headless
```
`pb_emu_resmode=3` is an 828x1200 window; the default is the device's real 1072x1448, which X11
will not scale down to fit a laptop screen. See the generated project's own README for the details.

**NOTE:** on macOS, `debug` builds of the `slint` variant need a raised soft limit for open file
descriptors. The generated `build` recipe raises it to 4096 itself, so no manual `ulimit -n 4096` is
required.
