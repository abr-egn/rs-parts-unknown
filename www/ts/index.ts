import "reflect-metadata";
import {container} from "tsyringe";
import * as ReactDOM from "react-dom";

import {Game} from "./game";
import {Base} from "./states";
import {index} from "../tsx/index";

declare global {
  interface Window {
    game: Game;
  }
}

import('../wasm').then(rust => {
  ReactDOM.render(index, document.getElementById("root"));

  const backend = new rust.PartsUnknown();
  const game = new Game(backend);
  window.game = game;
  container.register(Game, {useValue: game});
  game.stack.push(new Base());
}).catch(console.error);