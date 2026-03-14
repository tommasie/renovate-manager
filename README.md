# Renovate Manager
Renovate Manager is a TUI program aimed at easily reviewing all the [Renovate](https://docs.renovatebot.com/) pull-requests for which the user has merge approval rights.

## User guide
**WIP**

## Building the project
**WIP**
### Standalone Rust toolchain
`cargo build`
### The Nix way
The project supports a Nix shell containing the Rust toolchain and all the needed tools, defined in the [flake file](./flake.nix).

To create a shell with all the tooling run `nix develop`
