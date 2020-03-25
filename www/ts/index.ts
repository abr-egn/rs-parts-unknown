import {createCheckers} from "ts-interface-checker";
import dataTI from "./data-ti";

const {Display} = createCheckers(dataTI);

import('../wasm').then(rust => {
  console.log(rust.add(1, 2));

  let game = new rust.PartsUnknown();
  let display = game.get_display();
  Display.check(display);
  console.log(display);
}).catch(console.error);