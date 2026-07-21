# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this repository is

This is **not** a Rust project — it is a `cargo-generate` template that produces PocketBook e-reader
applications built on [inkview-rs](https://github.com/simmsb/inkview-rs). Nothing here compiles in
place; `template/` is expanded through Liquid templating into a new crate first.

One template covers three mutually exclusive UI stacks, selected by the `framework` placeholder:

| `framework` | Crates pulled in | Entry point |
| --- | --- | --- |
| `none` | `inkview` only | raw FFI calls (`OpenFont`, `DrawTextRect`, `FullUpdate`) |
| `slint` | `inkview-slint`, `slint`, `euclid`, `rgb` | Slint platform backend + `ui/main.slint` |
| `embedded-graphics` | `inkview-eg`, `embedded-graphics`, `anyhow` | `InkviewDisplay` as a `DrawTarget` |

## Generating and testing a template

```bash
# From a local checkout (what CI does)
cargo generate --path . template --name test-slint --define framework=slint --destination /tmp
cd /tmp/test-slint && just build

# From the published repo
cargo generate --git https://github.com/ihrfv/inkview-rs-templates.git template \
  --name <PROJECT_NAME> --define framework=<none|slint|embedded-graphics>
```

The generated project needs Nix + direnv + devenv (it ships `devenv.nix`/`.envrc`), which supply
`zig`, `cargo-zigbuild`, `just`, and the cross `pkg-config`/fontconfig search paths.

There are no unit tests. The only verification is "does each of the three frameworks generate and
cross-compile" — see `.github/workflows/ci.yml`, which matrixes over all three inside a devenv shell.
Always check all three variants after touching shared files; a Liquid or Cargo.toml change that
works for `none` can easily break `slint`.

## Editing the template — Liquid vs. `just`/Slint syntax

`cargo-generate` runs Liquid over every file not excluded by `[template.syntax].skip` in
`cargo-generate.toml`. Two consequences dominate day-to-day edits:

- **`template/justfile` uses `{{ }}` for its own variable interpolation**, which collides with
  Liquid. The whole recipe body is wrapped in `{% raw %} … {% endraw %}`. Any `{{crate_name}}`
  substitution must live *above* the `{% raw %}` block (see the variable declarations at the top of
  the file); anything added inside the raw block is emitted verbatim.
- Framework-specific content is selected with `{% if framework == "slint" %}` /
  `{% elsif … %}` / `{% else %}` blocks inside a *single* file — notably
  `template/src/bin/{{crate_name}}.rs`, which contains all three complete demo programs, and
  `template/Cargo.toml`. Filenames themselves are templated too (`{{crate_name}}.rs`).
- `[conditional.'framework != "slint"']` deletes `ui/` and `build.rs` from non-Slint projects, so
  those two files never need Liquid guards.

## Build/deploy model of the generated project

`template/justfile` is the whole interface. Its top-of-file variables are the intended knobs and are
overridden on the command line (`just cargo_profile=release pb_device=PB632 deploy-usb`):

- Cross-compilation is always `cargo zigbuild --target armv7-unknown-linux-gnueabi.2.23` — the
  glibc-version suffix is what makes the binary run on the reader's old userspace. `cargo build`
  alone will not produce a working artifact.
- `pb_sdk_version` maps to a Cargo feature (`6.10` → `sdk-6-10`) that selects the `inkview` FFI
  bindings; `Cargo.toml` defaults to `sdk-6-10`.
- `deploy-usb` copies to the mounted device volume; `deploy-ssh` scp's to a `.stage` path then
  atomically renames, kills, and relaunches the app. `build-deploy-ssh` chains both.
- `preconfigure-build-and-deploy-ssh` raises `ulimit -n` and loads the SSH key. On macOS, debug
  builds of the Slint variant fail without `ulimit -n 4096`.

`template/src/lib.rs` exists (currently empty) so generated projects have a lib+bin layout; the
binary is built with `--bin {{crate_name}}`.
