# BVE-Reborn

BVE-Reborn is a remake of the train simulator OpenBVE that focuses on visual quality and
performance, as well as code quality and flexibility.

While progress is strong, there is still a lot of work to do in order to get a working
demo.

BVE uses Rust for all code, which allows the code to be robust and safe from crashes
while being just as fast as C/C++.

## Building from Source

Binaries will be provided when there is a release, but for now, only developers can
make use of BVE-Reborn. If you are a developer the following is how you build from source.

### Clone

There are a couple submodules, so recursive cloning is required.

```
git clone https://github.com/BVE-Reborn/bve-reborn.git --recursive
```

If you've already cloned:

```
git submodule update --init --recursive
```

### Rust toolchain

You need to install the 2020-03-19 nightly toolchain of rust:

```
rustup install nightly-2020-06-22
```

Then you may run the main build process:

```
cargo run --bin bve-build --release
```

This will build bve, generate C/C++ bindings, and build the game.
