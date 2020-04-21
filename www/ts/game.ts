import * as ReactDOM from "react-dom";
import * as React from "react";

import * as wasm from "../wasm";
import {Id} from "../wasm";

import {GameBoard} from "./game_board";
import * as stack from "./stack";

import {Index} from "../tsx/index";

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

    async updateWorld(events: wasm.Event[], world: wasm.World, preview: (ev: wasm.Event) => void) {
        await this._board.animateEvents(events, preview);
        this._world.free();
        this._world = world;
        this._board.updateWorld(this._world);

        this._onUpdate();
    }

    // Private

    private _onUpdate() {
        let element = React.createElement(Index, {
            world: this._world,
            data: this._stack.data(),
            intents: this._getIntents(),
        }, null);
        ReactDOM.render(element, document.getElementById("root"));
    }

    private _getIntents(): [Id<wasm.Creature>, wasm.NPC, DOMPointReadOnly][] {
        const intents: [Id<wasm.Creature>, wasm.NPC, DOMPointReadOnly][] = [];
        for (let id of this._world.getCreatureIds()) {
            const point = this._board.creatureCoords(id);
            if (!point) { continue; }
            const creature = this._world.getCreature(id);
            if (!creature) { continue; }
            if (creature.dead) { continue; }
            let intent = creature.npc;
            if (!intent) { continue; }
            intents.push([id, intent, point]);
        }
        return intents;
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