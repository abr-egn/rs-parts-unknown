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
    public keys: Map<string, boolean> = new Map();
    constructor() {
        this._world = new World();
        this._stack = new Stack();

        window.game = this;

        let [content, ref] = index();
        ReactDOM.render(content, document.getElementById("root"));
        this._index = ref;

        const canvas = document.getElementById("mainCanvas") as HTMLCanvasElement;
        this._render = new Render(canvas, this._world, this._stack);

        canvas.focus();
        canvas.addEventListener('keydown', (e) => {
            e.shiftKey
            this.keys.set(e.code, true);
        });
        canvas.addEventListener('keyup', (e) => {
            this.keys.set(e.code, false);
        });
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

    getUI<T>(key: StateKey<T>): (T & StateUI) | undefined {
        return this._index.current?.getStack(key);
    }

    // Mutators

    updateWorld(world: World) {
        this._world.free();
        this._world = world;
        this._render.world = this._world;
        this._index.current!.setWorld(this._world);
    }

    updateUI<T>(key: StateKey<T>, update: (draft: T & StateUI) => void) {
        this._index.current!.updateStack(key, update);
    }
}