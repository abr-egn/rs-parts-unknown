import {RefObject} from "react";

import {World} from "../wasm";
import {Render} from "./render";
import {Stack, StateKey, StateUI} from "./stack";
import {Index} from "../tsx/index";
import {Hex, Tile} from "./types";

export class Game {
    private _world: World;
    private _stack: Stack;
    private _render: Render;
    constructor(private _index: RefObject<Index>) {
        this._world = new World();
        this._stack = new Stack();
        this._render = new Render(
            document.getElementById("mainCanvas") as HTMLCanvasElement,
            this._world, this._stack);
        this._index.current!.setWorld(this._world);
    }

    // Accessors

    get world(): World {
        return this._world;
    }

    get render(): Render {
        return this._render;
    }

    get stack(): Stack {
        return this._stack;
    }

    tileAt(hex: Hex): Tile | undefined {
        return this._world.getTile(hex);
    }

    // Mutators

    set index(value: RefObject<Index>) {
        this._index = value;
    }

    updateWorld(world: World) {
        this._world.free();
        this._world = world;
        this._render.world = this._world;
        this._index.current!.setWorld(this._world);
    }

    updateUI<T extends StateUI>(key: StateKey<T>, update: (draft: T) => void) {
        this._index.current!.updateStack(key, update);
    }
}