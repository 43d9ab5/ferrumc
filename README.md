# FerrumC

![GitHub License](https://img.shields.io/github/license/Sweattypalms/ferrumc)
![GitHub code size in bytes](https://img.shields.io/github/languages/code-size/Sweattypalms/ferrumc)
![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/Sweattypalms/ferrumc/rust.yml)
![Lines of code](https://www.aschey.tech/tokei/github.com/Sweattypalms/ferrumc)

A high performance Minecraft server written in Rust.

## Description

FerrumC is a Minecraft server written from the ground up in rust with performance in mind. Utilizing the power of the
Rust language, FerrumC is able to achieve high performance and low latency. FerrumC is designed to be a drop-in
replacement for the vanilla Minecraft server, and aims to be compatible with all vanilla clients. The open-source
nature of FerrumC allows for the community to contribute to the project and help shape the future of the server. It
also allows for easy tweaking and customization of the server to fit the needs of the user.

## Getting Started

### Dependencies

* Nothing but the Rust compiler!

### Installing

You can either download the source code and compile it yourself to get bleeding edge updates, or download a pre-compiled
binary from the releases page.

### Using FerrumC

#### Compiling from source

```bash
git clone https://github.com/Sweattypalms/ferrumc
cd ferrumc
cargo build --release # This will take a while
cd target/release
```

#### Running the server

* Move the binary to a directory of your choice (it will create a few files in the directory it is run from)
* Run the binary with `./ferrumc.exe` on Windows or `./ferrumc` on Linux/Mac to start up the server with the default
  settings
* You can also run `./ferrumc --setup` to generate a config file and then edit it to your liking

## License

This project is licensed under the MIT License - see the LICENSE.md file for details

## Acknowledgments

* The madlads behind the [Tokio Runtime](https://github.com/tokio-rs/tokio) for basically everything
* The amazing people working on [wiki.vg](https://wiki.vg) for the incredible amount of information it provides
* The [fastanvil](https://crates.io/crates/fastanvil), [simdnbt](https://github.com/azalea-rs/simdnbt) and
  [fastnbt](https://crates.io/crates/fastnbt) crates for saving me from the pain of decoding NBT and anvil data on my
  own
* The kind denizens of multiple discord server for answering my stupid questions and helping me out with stuff I don't
  understand