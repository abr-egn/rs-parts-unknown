import * as React from "react";
import * as ReactDOM from "react-dom";

import {FloatText} from "../tsx/float";
import {Index} from "../tsx/index";
import * as wasm from "../wasm";
import {GameBoard} from "./game_board";
import * as stack from "./stack";
import {Preview} from "./stack/preview";

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
    private _board: GameBoard;
    private _keys: Map<string, boolean> = new Map();
    private _floats: FloatText.ItemSet = new FloatText.ItemSet();
    constructor() {
        this._onUpdate = this._onUpdate.bind(this);

        this._world = new wasm.World();
        this._world.setTracer(new ConsoleTracer());

        this._stack = new stack.Stack(this._onUpdate);

        const canvas = document.getElementById("mainCanvas") as HTMLCanvasElement;
        this._board = new GameBoard(canvas, this._world, this._stack.boardListener(), this._stack.data());

        canvas.focus();
        canvas.addEventListener('keydown', (e) => {
            this._keys.set(e.code, true);
        });
        canvas.addEventListener('keyup', (e) => {
            this._keys.set(e.code, false);
        });

        // Force a render to ensure everything's populated.
        this._onUpdate();

        window.game = this;
    }

    // Accessors

    get world(): wasm.World {
        return this._world;
    }

    get stack(): stack.Stack {
        return this._stack;
    }

    get board(): GameBoard.View {
        return this._board;
    }

    key(name: string): boolean {
        return this._keys.get(name) || false;
    }

    creatureAt(hex: wasm.Hex): wasm.Creature | undefined {
        const tile = this._world.getTile(hex);
        if (!tile) { return undefined; }
        const id = tile.creature;
        if (id == undefined) { return undefined; }
        return this._world.getCreature(id);
    }

    // Mutators

    updateWorld(newWorld: wasm.World) {
        this._world.free();
        this._world = newWorld;
        this._board.updateWorld(this._world);
        this._onUpdate();
    }

    addFloat(float: FloatText.Item) {
        this._floats.add(float);
        this._onUpdate();
    }

    deleteFloat(float: FloatText.Item) {
        this._floats.delete(float);
        this._onUpdate();
    }

    // Private

    private _onUpdate() {
        let element = React.createElement(Index, {
            world: this._world,
            data: this._stack.data(),
            intents: this._getIntents(),
            floats: this._getFloats(),
        }, null);
        ReactDOM.render(element, document.getElementById("root"));
    }

    private _getIntents(): [wasm.Creature, DOMPointReadOnly][] {
        const intents: [wasm.Creature, DOMPointReadOnly][] = [];
        for (let creature of this._world.getCreatures()) {
            if (creature.dead || !creature.npc) { continue; }
            const point = this._board.creatureCoords(creature.id);
            if (!point) { continue; }
            intents.push([creature, point]);
        }
        return intents;
    }

    private _getFloats(): FloatText.ItemId[] {
        const data = this._stack.data();
        const floats: FloatText.ItemId[] = [];
        const prevFloats = data.get(Preview)?.float;
        if (prevFloats) { floats.push(...prevFloats); }
        floats.push(...this._floats.all);
        return floats;
    }
}

export class ConsoleTracer implements wasm.Tracer {
    resolveAction(action: wasm.Action, events: [wasm.Event]) {
        console.log("ACTION:", action);
        for (let event of events) {
            console.log("==>", event);
        }
    }
    systemEvent(events: [wasm.Event]) {
        console.log("SYSTEM:", events[0]);
        for (let event of events.slice(1)) {
            console.log("==>", event);
        }
    }
}

export class BufferTracer implements wasm.Tracer {
    private _buffer: (() => void)[] = [];
    constructor(private _wrapped: wasm.Tracer) {}
    resolveAction(action: wasm.Action, events: [wasm.Event]) {
        this._buffer.push(() => this._wrapped.resolveAction(action, events));
    }
    systemEvent(events: [wasm.Event]) {
        this._buffer.push(() => this._wrapped.systemEvent(events));
    }
    runBuffer() {
        for (let thunk of this._buffer) {
            thunk();
        }
    }
}