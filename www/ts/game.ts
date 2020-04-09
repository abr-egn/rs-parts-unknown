import {RefObject} from "react";
import * as ReactDOM from "react-dom";

import {
    Event, Tracer, World,
    findBoundary,
} from "../wasm";
import {Render} from "./render";
import {Stack} from "./stack";
import {Index, index} from "../tsx/index";

declare global {
    interface Window {
        game: Game;
        findBoundary: any;
    }
}

Window.prototype.findBoundary = findBoundary;

export class Game {
    private _world: World;
    private _stack: Stack;
    private _index: RefObject<Index>;
    private _render: Render;
    public keys: Map<string, boolean> = new Map();
    constructor() {
        this._world = new World();
        this._world.setTracer(new ConsoleTracer());
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

    get index(): Index {
        return this._index.current!;
    }

    // Mutators

    updateWorld(world: World) {
        this._world.free();
        this._world = world;
        this._render.world = this._world;
        this._index.current!.setWorld(this._world);
    }
}

export class ConsoleTracer implements Tracer {
    startAction(action: any) {
        console.log("ACTION:", action);
    }
    modAction(name: string, prev: any, new_: any) {
        console.log(" [%s]", name, prev, "->", new_);
    }
    resolveAction(action: any, event: Event) {
        console.log("==>", event);
    }
}

export class BufferTracer implements Tracer {
    private _buffer: (() => void)[] = [];
    constructor(private _wrapped: Tracer) {}
    startAction(action: any) {
        this._buffer.push(() => this._wrapped.startAction(action));
    }
    modAction(name: string, prev: any, new_: any) {
        this._buffer.push(() => this._wrapped.modAction(name, prev, new_));
    }
    resolveAction(action: any, event: Event) {
        this._buffer.push(() => this._wrapped.resolveAction(action, event));
    }
    runBuffer() {
        for (let thunk of this._buffer) {
            thunk();
        }
    }
}