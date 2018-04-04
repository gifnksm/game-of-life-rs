# Conway's Game of Life implementation for Native/Web

This game runs on both the web and native!

[Try in your browser](https://gifnksm.github.io/game-of-life-rs/)

## How to Build for Native

1. Install SDL2

  * Arch Linux:

    ```
    sudo pacman -S sdl2
    ```

  * macOS (Homebrew):

    ```
    brew install sdl2
    ```

  * Windows (MSYS2):

    ```
    pacman -S mingw-w64-SDL2
    ```

2. Build and Run

   ```
   git clone https://github.com/gifnksm/game-of-life-rs.git
   cd game-of-life-rs

   export LIBRARY_PATH=<path_to_sdl2> # optional
   cargo run --release
   ```

## How to Build for the Web

You should read the following post before start building:
https://users.rust-lang.org/t/compiling-to-the-web-with-rust-and-emscripten/7627

1. Install Emscripten SDK

   ```
   curl -O https://s3.amazonaws.com/mozilla-games/emscripten/releases/emsdk-portable.tar.gz
   tar -xzf emsdk-portable.tar.gz
   source emsdk_portable/emsdk_env.sh
   emsdk update
   emsdk install sdk-incoming-64bit
   emsdk activate sdk-incoming-64bit
   ```

2. Install Rust Standard Library for asm.js

   ```
   rustup target add asmjs-unknown-emscripten --toolchain nightly
   ```

3. Prepare Emscripten Caches

   ```
   # Set envvars for Emscripten toolchains
   source emsdk_portable/emsdk_env.sh

   echo "int main(void) { return 0; }" > empty.c
   emcc -s USE_SDL=2 empty.c     # for debug builds
   emcc -s USE_SDL=2 -O3 empty.c # for release builds
   ```

   Without this step, `cargo` build may fail.

   I think this is because `cargo` runs `emcc` in parallel.
   `emcc` does a lazy compile of dependencies (libc, gl, SDL2, ...) the first time you run it.
   If `cargo` runs `emcc` in parallel, each `emcc` process starts compiling dependencies and
   seems to write the build artifacts on the same path at the same time.
   This makes the build broken.

4. Build and Launch Web Server

   ```
   git clone https://github.com/gifnksm/game-of-life-rs.git
   cd game-of-life-rs

   # asmjs target is available only for nightly so far.
   rustup toolchain add nightly
   rustup override set nightly

   # Set envvars for Emscripten toolchains
   source emsdk_portable/emsdk_env.sh

   # Build and serve
   make
   make serve
   ```

5. Access to the Following URL

   ```
   http://localhost:8080/
   ```
