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
