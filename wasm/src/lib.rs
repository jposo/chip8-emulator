use chip8_core::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, KeyboardEvent};
use js_sys::Uint8Array;

// this tag tells the compiler that this struct needs to be configured for wasm
// any function or struct that is going to be called from JS will need to have it
#[wasm_bindgen]
pub struct EmuWasm {
    chip8: Emu,
    ctx: CanvasRenderingContext2d,
}

#[wasm_bindgen]
impl EmuWasm {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<EmuWasm, JsValue> {
        let chip8 = Emu::new();
        // get html DOM
        let document = web_sys::window().unwrap().document().unwrap();
        // get canvas by id
        let canvas = document.get_element_by_id("canvas").unwrap();
        let canvas: HtmlCanvasElement = canvas
            .dyn_into::<HtmlCanvasElement>()
            .map_err(|_| ())
            .unwrap();
        // get canvas' context
        let ctx = canvas.get_context("2d")
            .unwrap().unwrap()
            .dyn_into::<CanvasRenderingContext2d>()
            .unwrap();

        Ok(EmuWasm{chip8, ctx})
    }
    // the following functions are prettyt simple
    // just calling upon the functions that are in chip8_core
    #[wasm_bindgen]
    pub fn tick(&mut self) {
        self.chip8.tick();
    }

    #[wasm_bindgen]
    pub fn tick_timers(&mut self) {
        self.chip8.tick_timers();
    }
    // reset function that we never used in chip8_core, until now
    #[wasm_bindgen]
    pub fn reset(&mut self) {
        self.chip8.reset();
    }
    // keyboard inputs received directly from javascript
    #[wasm_bindgen]
    pub fn keypress(&mut self, evt: KeyboardEvent, pressed: bool) {
        let key = evt.key();
        if let Some(k) = key2btn(&key) {
            self.chip8.keypress(k, pressed);
        }
    }
    // receives and handles a javascript object
    #[wasm_bindgen]
    pub fn load_game(&mut self, data: Uint8Array) {
        self.chip8.load(&data.to_vec());
    }
    // rendering the screen
    // to render to the html5 canvas:
    // o obtain the canvas object and its context (object which gets draw functions upon it)
    // o change the /new/ constructor to grab the current window, canvas, and context (like js)
    #[wasm_bindgen]
    pub fn draw_screen(&mut self, scale: usize) {
        let disp = self.chip8.get_display();
        // iterate through every display's pixel
        for i in 0..(SCREEN_WIDTH * SCREEN_HEIGHT) {
            // if it is supposed to be white
            if disp[i] {
                let x = i % SCREEN_WIDTH;
                let y = i / SCREEN_WIDTH;
                // draw it to the screen
                self.ctx.fill_rect(
                    (x * scale) as f64,
                    (y * scale) as f64,
                    scale as f64,
                    scale as f64
                );
            }
        }
    }
}
// similar implementation done in desktop, except with JS keyboard events
fn key2btn(key: &str) -> Option<usize> {
    match key {
        "1" => Some(0x1),
        "2" => Some(0x2),
        "3" => Some(0x3),
        "4" => Some(0xC),
        "q" => Some(0x4),
        "w" => Some(0x5),
        "e" => Some(0x6),
        "r" => Some(0xD),
        "a" => Some(0x7),
        "s" => Some(0x8),
        "d" => Some(0x9),
        "f" => Some(0xE),
        "z" => Some(0xA),
        "x" => Some(0x0),
        "c" => Some(0xB),
        "v" => Some(0xF),
        _   => None,
    }
}