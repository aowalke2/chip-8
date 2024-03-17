use core::*;
use js_sys::Uint8Array;
use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, KeyboardEvent};

#[wasm_bindgen]
pub struct InterpreterWasm {
    chip8: Interpreter,
    context: CanvasRenderingContext2d,
}

#[wasm_bindgen]
impl InterpreterWasm {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<InterpreterWasm, JsValue> {
        let chip8 = Interpreter::new();

        let document = web_sys::window().unwrap().document().unwrap();
        let canvas = document.get_element_by_id("canvas").unwrap();
        let canvas: HtmlCanvasElement = canvas
            .dyn_into::<HtmlCanvasElement>()
            .map_err(|_| ())
            .unwrap();

        let context = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()
            .unwrap();

        Ok(InterpreterWasm { chip8, context })
    }

    #[wasm_bindgen]
    pub fn tick(&mut self) {
        self.chip8.tick();
    }

    #[wasm_bindgen]
    pub fn tick_timers(&mut self) {
        self.chip8.tick_timers();
    }

    #[wasm_bindgen]
    pub fn reset(&mut self) {
        self.chip8.reset();
    }

    #[wasm_bindgen]
    pub fn keypress(&mut self, event: KeyboardEvent, pressed: bool) {
        let key = event.key();
        if let Some(k) = match_key_to_btn(&key) {
            self.chip8.keypress(k, pressed);
        }
    }

    #[wasm_bindgen]
    pub fn load(&mut self, data: Uint8Array) {
        self.chip8.load(&data.to_vec());
    }

    #[wasm_bindgen]
    pub fn draw_screen(&mut self, scale: usize) {
        let display = self.chip8.get_screen();
        for row in 0..display.len() {
            for col in 0..display[0].len() {
                if display[row][col] {
                    self.context.fill_rect(
                        (col * scale) as f64,
                        (row * scale) as f64,
                        scale as f64,
                        scale as f64,
                    );
                }
            }
        }
    }
}

fn match_key_to_btn(key: &str) -> Option<usize> {
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
        _ => None,
    }
}
