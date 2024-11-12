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

## Windows Powershell
First, run the following commands to build with tectonic backend equipped.
```powershell
> [System.Environment]::SetEnvironmentVariable('TECTONIC_DEP_BACKEND','vcpkg')
> [System.Environment]::SetEnvironmentVariable('VCPKGRS_TRIPLET','x64-windows-static-release')
> [System.Environment]::SetEnvironmentVariable('RUSTFLAGS','-Ctarget-feature=+crt-static')
> [System.Environment]::SetEnvironmentVariable('VCPKG_ROOT','<current-git-menifest-path>\target\vcpkg')
> cargo vcpkg build
> cargo build --features=tectonic-backend
```
where `<current-git-menifest-path>` is the path where this git repo is cloned.

If you want to install, replace the last command with the following:
```powershell
> cargo install --path . --features=tectonic-backend
```

If one does not want to use tectonic backend but using local LaTeX compiler to
compile vesti, see [the main vesti repo](https://github.com/e0328eric/vesti).
