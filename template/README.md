# Overview

## Set-up

Similarly to [inkview-rs](https://github.com/simmsb/inkview-rs) the project set up relies on:
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

## Deploying

There are various ways of deploying the application (see [inkview-rs](https://github.com/simmsb/inkview-rs) for different depoyment strategies).

Meanwhile, one can addapt to their needs the script `./pb_build_and_deploy_ssh.sh` and call it.
The script does the following:
1. Increases the process's soft limit for the number of open file descriptors;
1. Build in `release` mode the application
1. Registers the ssh key necessary for the connection
1. Deploys it via `scp` and `ssh` to the device.
The application won't show up immediately - instead it will start up and you should select in in the list of openned apps (hold the home button for 3sec).
