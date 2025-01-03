# vesti-tectonic-backend

I don't know why that the tectonic stable backend does not compiled. But at
least, for recent tectonic branch it does compile.

This repo is purposed to compile vesti with tectonic backend. I tested it on
MSYS2 sandbox and powershell on Windows 11.

# Installation
## Prerequisits
In the tectonic book, it recommends to use [cargo-vcpkg](https://crates.io/crates/cargo-vcpkg) to build it.
One can install using the following command
```powershell
> cargo install cargo-vcpkg
```
Also, python 3.12 is need to use vesti

## Windows Powershell
Just run the batch file to install.

## Linux and Macos
On linux, just install following libraries which tectonic requires
- fontconfig (except on macOS)
- freetype2
- graphite2
- harfbuzz
- ICU4C
- libpng
- zlib

These list is presented in the [tectonic book](https://tectonic-typesetting.github.io/book/latest/howto/build-tectonic/external-dep-install.html).

Then run this command to build
```console
$ cargo build --features=tectonic-backend
```
or this one to install
```console
$ cargo install --path . --features=tectonic-backend
```

# Notes
If one does not want to use tectonic backend but using local LaTeX compiler to
compile vesti, see [the main vesti repo](https://github.com/e0328eric/vesti).
