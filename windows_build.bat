@echo off
REM Set environment variables
set TECTONIC_DEP_BACKEND=vcpkg
set VCPKGRS_TRIPLET=x64-windows-static-release
set RUSTFLAGS=-Ctarget-feature=+crt-static
set VCPKG_ROOT=%CD%\target\vcpkg

REM Execute the binary
cargo vcpkg build
cargo install --path . --features=tectonic-backend

REM Optional: Pause to see the output before closing
pause

