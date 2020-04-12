import produce from "immer";

import {
    Event, Tracer, World,
    findBoundary,
} from "../wasm";
import * as render from "./render";
import * as stack from "./stack";
import {renderIndex} from "../tsx/index";
import {UiData} from "./ui_data";

declare global {
    interface Window {
        game: Game;
        findBoundary: any;
    }
}

Window.prototype.findBoundary = findBoundary;

/*
This:
    a. Initializes everything and sets up linkages, and
    b. Is the interface through which the UI and the game states act.

Re. (b), it is made available as a global because there would be no gain to
manually threading it through to all places.
*/
export class Game {
    private _world: World;
    private _stack: stack.Stack;
    private _data: UiData;
    private _oldData: UiData[] = [];
    private _render: render.Render;
    private _keys: Map<string, boolean> = new Map();
    constructor() {
        this._world = new World();
        this._world.setTracer(new ConsoleTracer());

        this._data = new UiData();

        const stackData: stack.DataUpdate = {
            update: (update) => {
                this._data = produce(this._data, update);
                renderIndex(this._world, this._data);
            },
        };
        const stackListener: stack.Listener = {
            prePush: () => {
                this._oldData.push(this._data);
            },
            postPop: () => {
                this._data = this._oldData.pop()!;
                renderIndex(this._world, this._data);
            },
        };
        this._stack = new stack.Stack(stackData, stackListener);

        renderIndex(this._world, this._data);

        const canvas = document.getElementById("mainCanvas") as HTMLCanvasElement;
        const renderData: render.DataQuery = {
            get: (key) => { return this._data.get(key); },
        };
        this._render = new render.Render(canvas, this._world, this._stack, renderData);

        canvas.focus();
        canvas.addEventListener('keydown', (e) => {
            this._keys.set(e.code, true);
        });
        canvas.addEventListener('keyup', (e) => {
            this._keys.set(e.code, false);
        });

        window.game = this;
    }

    // Accessors

    get world(): World {
        return this._world;
    }

    get stack(): stack.Stack {
        return this._stack;
    }

    key(name: string): boolean {
        return this._keys.get(name) || false;
    }

    // Mutators

    async animateEvents(events: Event[]) {
        return this._render.animateEvents(events);
    }

    updateWorld(world: World) {
        this._world.free();
        this._world = world;
        this._render.updateWorld(this._world);

        renderIndex(this._world, this._data);
    }
}

export class ConsoleTracer implements Tracer {
    startAction(action: any) {
        console.log("ACTION:", action);
    }
    modAction(name: string, new_: any) {
        console.log(" [%s]", name, " ->", new_);
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
    modAction(name: string, new_: any) {
        this._buffer.push(() => this._wrapped.modAction(name, new_));
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