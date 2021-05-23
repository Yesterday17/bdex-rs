#!/usr/bin/env bash
# https://wapl.es/rust/2019/02/17/rust-cross-compile-linux-to-macos.html

MACOS_TARGET="x86_64-apple-darwin"

echo "Building target for platform ${MACOS_TARGET}"
echo

# Add osxcross toolchain to path
export PATH="$(pwd)/osxcross/target/bin:$PATH"

# Make libz-sys (git2-rs -> libgit2-sys -> libz-sys) build as a statically linked lib
# This prevents the host zlib from being linked
export LIBZ_SYS_STATIC=1

# Use Clang for C/C++ builds
export CC=o64-clang
export CXX=o64-clang++

# Use environmental variable instead of modifying cargo.toml
export CARGO_TARGET_X86_64_APPLE_DARWIN_LINKER=x86_64-apple-darwin14-clang
export CARGO_TARGET_X86_64_APPLE_DARWIN_AR=x86_64-apple-darwin14-ar

cargo build --release --target "${MACOS_TARGET}"

echo
echo Done