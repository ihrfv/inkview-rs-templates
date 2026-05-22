# Overview

## Set-up

Similarly to [inkview-rs](https://github.com/simmsb/inkview-rs/tree/master) the project set up relies on:
1. [Zig](https://ziglang.org/learn/getting-started/#installing-zig)
    Necessary for `cargo-zigbuild` which is used to properly perform linking process during.
1. [just](https://github.com/casey/just)
    Provides access to the helper commands already pre-configured for the use
1. [NIX](https://nixos.org/download/) + [direnv](https://direnv.net/docs/installation.html) + [devenv](https://devenv.sh/getting-started/).
    Nix installs a sandbox environment with the exact cross-compilation tools required for the e-reader, while direnv automatically injects them into your shell the moment you enter the project directory

The aformentioned components must be installed in order to simplify project compilation.

## Building the project

```bash
just build
```
