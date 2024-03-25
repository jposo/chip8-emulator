// when we compile with wasm-pack, it will generate the .wasm file..
// ..and a "glue" JS file that we can use here
import init, * as wasm from "./wasm.js"

const WIDTH = 64;
const HEIGHT = 32;
const SCALE = 15;
const TICKS_PER_FRAME = 10;
let anim_frame = 0;
// fetch the canvas object
const canvas = document.getElementById("canvas");
canvas.width = WIDTH * SCALE;
canvas.height = HEIGHT * SCALE;

const ctx = canvas.getContext("2d");
ctx.fillStyle = "black";
ctx.fillRect(0, 0, WIDTH * SCALE, HEIGHT * SCALE);

const input = document.getElementById("fileinput");

async function run() {
    // initalizes the wasm binary before using it
    await init();
    // create emulator backend object
    let chip8 = new wasm.EmuWasm();
    
    document.addEventListener("keydown", (evt) => {
        chip8.keypress(evt, true);
    });
    
    document.addEventListener("keyup", (evt) => {
        chip8.keypress(evt, false);
    });
    // handle file loading when file input button is clicked
    input.addEventListener("change", (evt) => {
        // stop previous game from rendering if one is running
        if (anim_frame != 0)
            window.cancelAnimationFrame(anim_frame);
        // get file path if it exists
        let file = evt.target.files[0];
        if (!file) {
            alert("Failed to read file");
            return;
        }
        // load in game as Uint8Array, send it to .wasm, start main loop
        let fr = new FileReader();
        fr.onload = (e) => {
            let buffer = fr.result;
            const rom = new Uint8Array(buffer);
            chip8.reset();
            chip8.load_game(rom);
            mainloop(chip8);
        }
        fr.readAsArrayBuffer(file);
    }, false);
}

function mainloop(chip8) {
    // only draw every few ticks
    for (let _ = 0; _ < TICKS_PER_FRAME; _++)
        chip8.tick();
    chip8.tick_timers();
    
    // clear the canvas before (re)drawing
    ctx.fillStyle = "black";
    ctx.fillRect(0, 0, WIDTH * SCALE, HEIGHT * SCALE);
    // set the draw color to white before rendering the frame
    ctx.fillStyle = "white";
    chip8.draw_screen(SCALE);
    // ensures 60 fps performance
    // restarts our mainloop when it is time
    anim_frame = window.requestAnimationFrame(() => {
        mainloop(chip8); // calls it again
    });
}

run().catch(console.error);