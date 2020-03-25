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

import('../wasm').then(rust => {
  let game = new rust.PartsUnknown();
  let display = asDisplay(game.get_display());
  console.log(display);
  let engine = new Render.Engine(document.getElementById("mainCanvas") as HTMLCanvasElement)
}).catch(console.error);