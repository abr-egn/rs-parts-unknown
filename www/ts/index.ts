import {enableAllPlugins} from "immer";

import "./types";
import {Game} from "./game";
import {Base} from "./states";

enableAllPlugins();

function main() {
  const game = new Game();
  game.stack.push(new Base());
}

main();