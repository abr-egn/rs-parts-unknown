import {createCheckers} from "ts-interface-checker";

import {Display} from "./data";
import dataTI from "./data-ti";
import * as Render from "./render";
import {Stack} from "./state";
import * as States from "./states";

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
    game: {engine: Render.Engine, stack: Stack};
  }
}

import('../wasm').then(rust => {
  const backend = new rust.PartsUnknown();
  const display = asDisplay(backend.get_display());
  const stack = new Stack();
  stack.push(new States.Base());
  const engine = new Render.Engine(
    document.getElementById("mainCanvas") as HTMLCanvasElement,
    display, stack);
  window.game = {engine, stack};
}).catch(console.error);