import {enableAllPlugins} from "immer";
import * as ReactDOM from "react-dom";
import "reflect-metadata";
import {container} from "tsyringe";

import {Game} from "./game";
import {Base} from "./states";
import {index} from "../tsx/index";

enableAllPlugins();

declare global {
  interface Window {
    game: Game;
  }
}

function main() {
  let [content, ref] = index();
  ReactDOM.render(content, document.getElementById("root"));

  const game = new Game(ref);
  container.register(Game, {useValue: game});
  game.stack.push(new Base());

  window.game = game;
}

main();