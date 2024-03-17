import init, * as wasm from "./wasm.js";

const WIDTH = 64;
const HEIGHT = 32;
const SCALE = 24;
const TICKS_PER_FRAME = 5;
let anim_frame = 0;

const canvas = document.getElementById("canvas");
canvas.width = WIDTH * SCALE;
canvas.height = HEIGHT * SCALE;

const context = canvas.getContext("2d");
context.fillStyle = "black";
context.fillRect(0, 0, WIDTH * SCALE, HEIGHT * SCALE);

const input = document.getElementById("fileinput");

async function run() {
  await init();
  let chip8 = new wasm.InterpreterWasm();

  document.addEventListener("keydown", function (event) {
    chip8.keypress(event, true);
  });

  document.addEventListener("keyup", function (event) {
    chip8.keypress(event, false);
  });

  input.addEventListener(
    "change",
    function (event) {
      if (anim_frame != 0) {
        window.cancelAnimationFrame(anim_frame);
      }

      let file = event.target.files[0];
      if (!file) {
        alert("Failed to read file");
        return;
      }

      let fileReader = new FileReader();
      fileReader.onload = function (e) {
        let buffer = fileReader.result;
        const rom = new Uint8Array(buffer);
        chip8.reset();
        chip8.load(rom);
        gameloap(chip8);
      };
      fileReader.readAsArrayBuffer(file);
    },
    false
  );
}

function gameloap(chip8) {
  for (let i = 0; i < TICKS_PER_FRAME; i++) {
    chip8.tick();
  }
  chip8.tick_timers();

  context.fillStyle = "black";
  context.fillRect(0, 0, WIDTH * SCALE, HEIGHT * SCALE);
  context.fillStyle = "white";
  chip8.draw_screen(SCALE);

  anim_frame = window.requestAnimationFrame(() => {
    gameloap(chip8);
  });
}

run().catch(console.error);
