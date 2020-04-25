import * as ReactDOM from "react-dom";
import * as React from "react";

import * as wasm from "../wasm";

import {hexToPixel} from "./draw";
import {GameBoard} from "./game_board";
import {Preview} from "./preview";
import * as stack from "./stack";

import {FloatText} from "../tsx/float";
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
    private _float: FloatText.ItemSet = new FloatText.ItemSet();
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
        await this._animateEvents(events, preview);
        this._world.free();
        this._world = world;
        this._board.updateWorld(this._world);

        this._onUpdate();
    }

    // Private

    private async _animateEvents(events: wasm.Event[], preview: (ev: wasm.Event) => void) {
        for (let event of events) {
            preview(event);
            let data;
            if (data = event.CreatureMoved) {
                await this._board.moveCreatureTo(data.id, hexToPixel(data.to))
            } else if (data = event.OnCreature) {
                let ce;
                if (ce = data.event.OnPart) {
                    let pe;
                    if (pe = ce.event.ChangeHP) {
                        const float = this._board.hpFloat(data.id, ce.id, pe.delta);
                        float.style!.animationName = "floatLift";
                        this._float.add(float);
                        this._onUpdate();
                        await new Promise(f => setTimeout(f, 2000));
                        this._float.delete(float);
                        this._onUpdate();
                    }
                }
            }
        }
    }

    private _onUpdate() {
        const floats = this._float.all;
        const prevFloats = this._stack.data().get(Preview)?.float;
        if (prevFloats) {
            floats.push(...prevFloats);
        }
        let element = React.createElement(Index, {
            world: this._world,
            data: this._stack.data(),
            intents: this._getIntents(),
            floats,
        }, null);
        ReactDOM.render(element, document.getElementById("root"));
    }

    private _getIntents(): [wasm.Creature, DOMPointReadOnly][] {
        const intents: [wasm.Creature, DOMPointReadOnly][] = [];
        for (let id of this._world.getCreatureIds()) {
            const point = this._board.creatureCoords(id);
            if (!point) { continue; }
            const creature = this._world.getCreature(id);
            if (!creature) { continue; }
            if (creature.dead) { continue; }
            if (!creature.npc) { continue; }
            intents.push([creature, point]);
        }
        return intents;
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