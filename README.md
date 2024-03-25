# chip8-emulator
Chip-8 Emulator created by following the guide made by [Toerktumlare](https://github.com/Toerktumlare/chip8-emulator).

Chip-8 games are in the `c8games` directory.

## Run locally
To run the emulator on your computer on binary, clone the repository, go to the repo path and run the following in a terminal:

`$ cd desktop`

`$ cargo run path/to/game`

Substitute `path/to/game` with your own game's path.

To run the emulator on the browser:

`$ cd wasm`

`$ wasm-pack build --target web`

Move the `wasm_bg.wasm` and `wasm.js` files inside the `pkg` directory into `web`.

Start a web server, or open `index.html`, inside `web`.

*still need to add audio*