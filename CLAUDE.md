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

There are no unit tests. Verification is "does each of the three frameworks generate, cross-compile,
and render a frame under the SDK emulator" — see `.github/workflows/ci.yml`, which matrixes over all
three inside a devenv shell. The emulator leg is the only run-time assertion in the repo: it fails
on a panic in the log and on a frame smaller than the emulated panel, and uploads the capture as an
artifact. Always check all three variants after touching shared files; a Liquid or Cargo.toml change
that works for `none` can easily break `slint`.

Locally, the same check is `just build-screenshot-emu` inside a generated project (see below).

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

## Desktop emulator in generated projects

Generated projects can run on the developer's machine instead of the device: the SDK ships a **host
x86_64 build of `libinkview.so`**, and since `inkview::load()` resolves the library by name at
runtime, a build for `x86_64-unknown-linux-gnu` against that copy runs the real app in an X11 window.

**The tooling is not in this repo.** It lives in
[inkview-rs-emu](https://github.com/ihrfv/inkview-rs-emu) — container definitions, the staging logic, the
XQuartz setup, the SDK matrix, and the `inkview-pilot` crate for tap/swipe/key input. It used to be
vendored into `template/utils/emulator/`; it was extracted so every inkview-rs project shares one
copy that can be updated without regenerating. Its README and CLAUDE.md are where the mechanism and
its constraints are documented — do not re-document them here, and check there first when something
emulator-shaped breaks.

What remains here is delegation:

- The `*-emu` recipes in `template/justfile` pass `pb_emu_*` through as `PB_EMU_*` environment
  variables and call `inkview-emu`. A `_require-emu` guard fails with install instructions rather
  than `command not found` halfway through a docker invocation.
- Emulator variables go **above** the `{% raw %}` block like every other justfile variable; the
  recipes interpolating them go inside it. Inside that block use `{{ binary }}`, **not**
  `{{ crate_name }}` — cargo-generate never substitutes inside `{% raw %}`, so `crate_name` would
  reach `just` as an undefined variable and fail at parse time.
- `.github/workflows/ci.yml` clones inkview-emu at a pinned ref onto `$GITHUB_PATH` and caches
  `~/.cache/inkview-emu`. Bump the ref deliberately; an unpinned clone would let an unrelated change
  there break this pipeline.
- `devenv.nix` gates the x86_64 cross pkg-config and fontconfig behind
  `{% if framework == "slint" %}` deliberately: only slint links C libraries, and merely referencing
  `pkgsCrossEmu.fontconfig.dev` forces a cross build that `none` and `embedded-graphics` users would
  pay for and never use. The `x86_64-unknown-linux-gnu` Rust target is unconditional, since all
  three variants build for it.
- **`template/Cargo.toml` carries a `[patch.crates-io]` for `inkview`, and it is temporary.**
  Published `inkview` 0.3.0 panics under the emulator (`Failed to get current task framebuffer`)
  because `GetTaskFramebuffer` returns NULL there; simmsb/inkview-rs#24 adds a `GetCanvas()`
  fallback but is unreleased, so the patch points at a branch on the fork carrying it. Verified by
  running: `framework=none` works without the patch (raw FFI, never constructs a `Screen`), while
  `slint` and `embedded-graphics` both panic inside `Screen::new`. Delete the section once a fixed
  release reaches crates.io.

## Settled questions — don't re-litigate these

Each of these was investigated at length on 2026-07-21. Re-deriving them is expensive.

- **The `armv7-unknown-linux-gnueabi` target is correct.** The PB632 is soft-float: its own
  binaries, `libinkview.so`, `libfontconfig.so.1`, and `/lib/ld-linux.so.3` all carry
  `e_flags 0x05000200`, identical to ours. It runs glibc 2.23, exactly matching the
  `.2.23` zigbuild suffix.
- **`devenv.nix` sourcing hard-float (`armv7l-hf-multiplatform`) cross libs is deliberate.**
  It looks wrong for a soft-float target, but those `.so`s only resolve symbols at link time —
  the device loads its own soft-float libraries at runtime. **Do not switch to a soft-float
  `crossSystem`**: it is in no binary cache and needs 37 derivations built from source (two GCC
  bootstraps, two glibc builds, then fontconfig's stack), turning a ~5 min CI leg into hours for
  zero runtime benefit.
- **The slint variant links `libfontconfig.so.1` at runtime and that is fine.** The device
  provides it, and all 24 `Fc*` symbols we import are exported by its (older) copy. The binary
  requires at most `GLIBC_2.18`.
- **`pb_ssh_ip` in the justfile is a placeholder, not a stale value.** The device is on DHCP;
  override per invocation (`just pb_ssh_ip="..." deploy-ssh`). Do not "fix" the checked-in IP.

## Working on this repo

- **Use the justfile recipes** (`just build`, `just deploy-ssh`) rather than hand-rolling the
  underlying `cargo zigbuild`/`scp` commands. The recipes are part of what is being tested;
  reimplementing them means debugging something the user does not ship.
- **CI failures reading `path '/nix/store/...' is not valid` are transient.** Re-run the job
  before investigating.
- **Large `scp` transfers to the device fail often** (~1 in 3 for an 8 MB binary), leaving no
  partial file. The cause is *unknown* — filename, memory pressure, quoting, and stdin were all
  hypothesised and all refuted. Do not infer a pattern from a handful of runs; retry, then
  confirm integrity with `md5sum` on device against local `md5`.
- **A green CI only means it compiles.** Runtime correctness needs a real device. As of
  2026-07-21 only the slint variant has been run-tested on hardware.
- Device userland is BusyBox: no `readelf`/`nm`/`file`, and its `od` rejects `-A`/`-j`/`-N`.
  Read ELF fields with `dd if=<file> bs=1 skip=36 count=4 | od -x`, or `scp` the file and
  inspect it locally. USB mounting exposes only `/mnt/ext1`, never the rootfs — use SSH for
  anything about system libraries.
