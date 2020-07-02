# BVE-Reborn

![[Discord](https://discord.gg/RxJHE2D)](https://img.shields.io/discord/451037457475960852?color=7289DA&label=discord&style=flat-square)
![Build Status](https://img.shields.io/github/workflow/status/BVE-Reborn/bve-reborn/build?label=build&style=flat-square)
![Release Status](https://img.shields.io/github/workflow/status/BVE-Reborn/bve-reborn/release?label=release&style=flat-square)
![Repo-State Status](https://img.shields.io/github/workflow/status/BVE-Reborn/bve-reborn/repo-state?label=repo-state&style=flat-square)
![Version](https://img.shields.io/github/v/release/BVE-Reborn/bve-reborn?include_prereleases&label=version&style=flat-square)

BVE-Reborn is a remake of the train simulator OpenBVE that focuses on visual quality and
performance, as well as code quality and flexibility.

While progress is strong, there is still a lot of work to do in order to get a working
demo.

BVE uses Rust for all code, which allows the code to be robust and safe from crashes
while being just as fast as C/C++.

## Building from Source

Binaries will be provided when there is a release, but for now, only developers can
make use of BVE-Reborn. If you are a developer the following is how you build from source.

### Rust toolchain

You need to install the 2020-06-22 nightly toolchain of rust:

```
rustup install nightly-2020-06-22
```

Then you may run the main build process:

```
cargo build  # Debug Build
cargo build --release  # Release build
```

This will build bve, generate C/C++ bindings, and build the game.

Running it requires data files, so contact me directly on the discord if you want to try building it.
