import {RefObject} from "react";
import * as ReactDOM from "react-dom";

import {World} from "../wasm";
import {Render} from "./render";
import {Stack, StateKey, StateUI} from "./stack";
import {Index, index} from "../tsx/index";
import {Hex, Tile, find_boundary} from "./types";

declare global {
    interface Window {
        game: Game;
        find_boundary: any;
    }
}

Window.prototype.find_boundary = find_boundary;

export class Game {
    private _world: World;
    private _stack: Stack;
    private _index: RefObject<Index>;
    private _render: Render;
    constructor() {
        this._world = new World();
        this._stack = new Stack();

        window.game = this;

        let [content, ref] = index();
        ReactDOM.render(content, document.getElementById("root"));
        this._index = ref;

        this._render = new Render(
            document.getElementById("mainCanvas") as HTMLCanvasElement,
            this._world, this._stack);
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