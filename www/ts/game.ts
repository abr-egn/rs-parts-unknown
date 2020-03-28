import {createCheckers} from "ts-interface-checker";

import {PartsUnknown} from "../wasm";

import {World, Hex, Meta, Tile} from "./data";
import dataTI from "./data-ti";
import {Render} from "./render";
import {Stack} from "./stack";
import * as States from "./states";

const CHECKERS = createCheckers(dataTI);

function checkWorld(world: any): World {
    try {
        CHECKERS.World.check(world);
        return world;
    } catch (err) {
        console.error(world);
        throw err;
    }
}

export class Game {
    private _stack: Stack;
    private _render: Render;
    private _map: Tile[][] = [];
    constructor(private _backend: PartsUnknown) {
        this._stack = new Stack();
        const world = checkWorld(this._backend.buildDisplay());
        this._render = new Render(
            document.getElementById("mainCanvas") as HTMLCanvasElement,
            world, this._stack);
        this._buildMap();
    }

    // Accessors

    get world(): World {
        return this._render.world;
    }

    get backend(): PartsUnknown {
        return this._backend;
    }

    get render(): Render {
        return this._render;
    }

    get stack(): Stack {
        return this._stack;
    }

    tileAt(hex: Hex): Tile | undefined {
        return this._map[hex.x]?.[hex.y];
    }

    // Mutators

    updateWorld() {
        const world = checkWorld(this._backend.buildDisplay());
        this._render.world = world;
        this._buildMap();
    }

    // Private

    private _buildMap() {
        this._map = [];
        for (let [hex, tile] of this.world.map) {
            if (this._map[hex.x] == undefined) {
                this._map[hex.x] = [];
            }
            this._map[hex.x][hex.y] = tile;
        }
    }
}