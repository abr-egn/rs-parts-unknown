import "reflect-metadata";
import {container} from "tsyringe";
import * as ReactDOM from "react-dom";

import {Game} from "./game";
import {Base} from "./states";
import {element} from "../tsx/first";

declare global {
  interface Window {
    game: Game;
  }
}

import('../wasm').then(rust => {
  const backend = new rust.PartsUnknown();
  const game = new Game(backend);
  window.game = game;
  container.register(Game, {useValue: game});
  game.stack.push(new Base());
  ReactDOM.render(element, document.getElementById("leftSide"));
}).catch(console.error);