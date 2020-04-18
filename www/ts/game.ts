import produce from "immer";

import * as wasm from "../wasm";
import {Id} from "../wasm";

import {GameBoard} from "./game_board";
import * as stack from "./stack";
import {UiData} from "./ui_data";

import {renderIndex} from "../tsx/index";

declare global {
    interface Window {
        game: Game;
    }
}

/*
This:
    a. Initializes everything and sets up linkages, and
    b. Is the interface through which the UI and the game states act.

Re. (b), it is made available as a global because there would be no gain to
manually threading it through to all places.
*/
export class Game {
    private _world: wasm.World;
    private _stack: stack.Stack;
    private _data: UiData;
    private _oldData: UiData[] = [];
    private _board: GameBoard;
    private _keys: Map<string, boolean> = new Map();
    constructor() {
        this._world = new wasm.World();
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
        const boardData: GameBoard.DataQuery = {
            get: (key) => { return this._data.get(key); },
        };
        this._board = new GameBoard(canvas, this._world, this._stack, boardData);

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

    get world(): wasm.World {
        return this._world;
    }

    get stack(): stack.Stack {
        return this._stack;
    }

    key(name: string): boolean {
        return this._keys.get(name) || false;
    }

    creatureCoords(id: Id<wasm.Creature>): DOMPointReadOnly | undefined {
        return this._board.creatureCoords(id);
    }

    // Mutators

    async animateEvents(events: wasm.Event[]) {
        return this._board.animateEvents(events);
    }

    updateWorld(world: wasm.World) {
        this._world.free();
        this._world = world;
        this._board.updateWorld(this._world);

        renderIndex(this._world, this._data);
    }
}

export class ConsoleTracer implements wasm.Tracer {
    startAction(action: any) {
        console.log("ACTION:", action);
    }
    modAction(name: string, new_: any) {
        console.log(" [%s]", name, " ->", new_);
    }
    resolveAction(action: any, events: [wasm.Event]) {
        for (let event of events) {
            console.log("==>", event);
        }
    }
}

export class BufferTracer implements wasm.Tracer {
    private _buffer: (() => void)[] = [];
    constructor(private _wrapped: wasm.Tracer) {}
    startAction(action: any) {
        this._buffer.push(() => this._wrapped.startAction(action));
    }
    modAction(name: string, new_: any) {
        this._buffer.push(() => this._wrapped.modAction(name, new_));
    }
    resolveAction(action: any, events: [wasm.Event]) {
        this._buffer.push(() => this._wrapped.resolveAction(action, events));
    }
    runBuffer() {
        for (let thunk of this._buffer) {
            thunk();
        }
    }
}