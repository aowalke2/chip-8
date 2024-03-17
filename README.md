# CHIP-8

In my quest to build a GBA emulator in rust, I started with a CHIP-8 interpreter in rust

## Running locally

Make sure you have the latest version of rust installed

### Mac OS

- Install [Homebrew](https://brew.sh/)
- Install sdl2 with homebrow: `brew install sdl2`
- Add this `export LIBRARY_PATH="$LIBRARY_PATH:$(brew --prefix)/lib"` to your .zshrc file if you use zsh or your .bashrc file if you use bash
- In root directory of this project do cargo run and point it at the chip8 rom file you would like to run. Ex. `cargo run /ROM_ADDRESS`

### Linux

- Install sdl2 with: `sudo apt update && sudo apt install -y libsdl2`
- Add this `export LIBRARY_PATH="$LIBRARY_PATH:$(brew --prefix)/lib"` to your .zshrc file if you use zsh or your .bashrc file if you use bash
- In root directory of this project do cargo run and point it at the chip8 rom file you would like to run. Ex. `cargo run /ROM_ADDRESS`

## WASM

- Make sure you have wasm-pack installed. If not use `cargo install wasm-pack`
- cd into the wasm directory of this project
- Run: `wasm-pack build --target web`
- Run: `mv pkg/wasm_bg.wasm ../web`
- Run: `mv pkg/wasm.js ../web`
- cd to the main project directory and then start a web server of you choice

## Sources

- [Cowgod's Chip-8 Technical Reference](http://devernay.free.fr/hacks/chip8/C8TECH10.HTM)
- [CHIP‐8 Technical Reference](https://github.com/mattmikolay/chip-8/wiki/CHIP%E2%80%908-Technical-Reference)
- [Chip8 Book](https://github.com/aquova/chip8-book)
- [Guide to making a CHIP-8 emulator](https://tobiasvl.github.io/blog/write-a-chip-8-emulator/)
- [How to write an emulator (CHIP-8 interpreter)](https://multigesture.net/articles/how-to-write-an-emulator-chip-8-interpreter/)
