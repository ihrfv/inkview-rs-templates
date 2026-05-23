# Overview

This repository is adjacent to [inkview-rs](https://github.com/simmsb/inkview-rs/tree/master).
It provides you with the templating mechanism to create new configured vanilla `inkview`, `inkview-eg`, and `inkview-slint` projects.

## Prerequisits
You must have `cargo-generate` installed. You can do it with
```bash
cargo install cargo-generate
```

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

To test that it works execute inside of the newely generate project:
```bash
just build
```

**NOTE:** on macOS it may fail to build `debug` release for the `slint` project.
To correct it, one should execute first increase the process's soft limit for the number of open file descriptors:
```bash
ulimit -n 4096
```
