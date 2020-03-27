import "reflect-metadata";
import {container} from "tsyringe";

import {Game} from "./game";

declare global {
  interface Window {
    game: Game;
  }
}

import('../wasm').then(rust => {
  const backend = new rust.PartsUnknown();
  const game = new Game(backend);
  container.register(Game, {useValue: game});

  window.game = game;
}).catch(console.error);