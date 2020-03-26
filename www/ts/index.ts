import {createCheckers} from "ts-interface-checker";

import {Display} from "./data";
import dataTI from "./data-ti";
import * as Render from "./render";

const CHECKERS = createCheckers(dataTI);

function asDisplay(display: any): Display {
  try {
    CHECKERS.Display.check(display);
    return display;
  } catch(err) {
    console.error(display);
    throw err;
  }
}

declare global {
  interface Window {
    game: {engine: Render.Engine};
  }
}

import('../wasm').then(rust => {
  const backend = new rust.PartsUnknown();
  const display = asDisplay(backend.get_display());
  const engine = new Render.Engine(
    document.getElementById("mainCanvas") as HTMLCanvasElement,
    display);
  window.game = {engine: engine};
}).catch(console.error);