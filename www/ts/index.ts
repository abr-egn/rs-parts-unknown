import "reflect-metadata";
import {container} from "tsyringe";

import {Game} from "./game";
import {Base} from "./states";

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
}).catch(console.error);