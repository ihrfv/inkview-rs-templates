# Overview

This repository is adjacent to [inkview-rs](https://github.com/simmsb/inkview-rs/tree/master).
It provides you with the templating mechanism to create new configured vanilla `inkview`, `inkview-eg`, and `inkview-slint` projects.

## Prerequisits
You must have `cargo-generate` installed. You can do it with
```bash
cargo install cargo-generate
```

## Creating a new project

At this point in time, the templating project is not yet published to crates.io, therefore, the only way to use it is to clone this repo first, and to invoke `cargo-generate` with a `--path` arg.

Here is the syntax:
```bash
cargo generate --path inkview-templates/ template --name <PROJECT_NAME> --define framework=<FRAMEWORK_TYPE>
```

`<FRAMEWORK_TYPE>` can be:
* `none` - for the vanilla `inkview` example project
* `slint` - for `inkview-slint` example project
* `embedded-graphics` - for `inkview-eg` example project

After creating the project, please update the project name in `Cargo.toml`
