import {createCheckers} from "ts-interface-checker";

import {PartsUnknown} from "../wasm";

import {Display, Hex, Tile} from "./data";
import dataTI from "./data-ti";
import * as Render from "./render";
import {Stack} from "./stack";
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

export class Game {
    private _stack: Stack;
    private _engine: Render.Engine;
    private _map: Tile[][] = [];
    constructor(private _backend: PartsUnknown) {
        this._stack = new Stack();
        this._stack.push(new States.Base());

        const display = asDisplay(this._backend.buildDisplay());
        this._engine = new Render.Engine(
            document.getElementById("mainCanvas") as HTMLCanvasElement,
            display, this._stack);
        this._buildMap();
    }

    get display(): Display {
        return this._engine.display;
    }

    get backend(): PartsUnknown {
        return this._backend;
    }

    get engine(): Render.Engine {
        return this._engine;
    }

    tileAt(hex: Hex): Tile | undefined {
        return this._map[hex.x]?.[hex.y];
    }

    private _buildMap() {
        this._map = [];
        for (let [hex, tile] of this.display.map) {
            if (this._map[hex.x] == undefined) {
                this._map[hex.x] = [];
            }
            this._map[hex.x][hex.y] = tile;
        }
    }
}