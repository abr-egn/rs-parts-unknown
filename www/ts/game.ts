import produce from "immer";

import {
    Event, Tracer, World,
    findBoundary,
} from "../wasm";
import {Render, DataQuery as RenderData} from "./render";
import {Stack, DataView as StackView, DataPush as StackPush} from "./stack";
import {renderIndex} from "../tsx/index";
import {UiData} from "./ui_data";

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
    private _data: UiData;
    private _render: Render;
    private _keys: Map<string, boolean> = new Map();
    constructor() {
        this._world = new World();
        this._world.setTracer(new ConsoleTracer());

        this._data = new UiData();

        const stackData: StackView & StackPush = {
            get: () => { return this._data; },
            set: (data) => {
                this._data = data;
                renderIndex(this._world, this._data);
            },
            update: (update) => {
                this._data = produce(this._data, update);
                renderIndex(this._world, this._data);
            }
        };
        this._stack = new Stack(stackData);

        renderIndex(this._world, this._data);

        const canvas = document.getElementById("mainCanvas") as HTMLCanvasElement;
        const renderData: RenderData = {
            get: (key) => { return this._data.get(key); },
        };
        this._render = new Render(canvas, this._world, this._stack, renderData);

        canvas.focus();
        canvas.addEventListener('keydown', (e) => {
            e.shiftKey
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

    get stack(): Stack {
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