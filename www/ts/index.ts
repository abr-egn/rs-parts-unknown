import {createCheckers} from "ts-interface-checker";
import dataTI from "./data-ti";

const {Display} = createCheckers(dataTI);

import('../wasm').then(rust => {
  let game = new rust.PartsUnknown();
  let display = game.get_display();
  console.log(display);
  Display.check(display);
}).catch(console.error);