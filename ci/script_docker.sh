#!/bin/bash

set -ev

source /emsdk/emsdk_env.sh

llvm-config --prefix
clang --version
emcc --version
rustc -vV
cargo -vV

TARGET=asmjs-unknown-emscripten

make
