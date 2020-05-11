import {immerable} from "immer";

import * as wasm from "../../wasm";
import {Id, Hex} from "../../wasm";

import {pathCreature, pathPart} from "../extra";
import {GameBoard} from "../game_board";
import {Stack, State} from "../stack";
import {Focus} from "../stack/focus";
import {Highlight} from "../stack/highlight";

import {FloatText} from "../../tsx/float";

export class LevelState extends State {
    // These live outside of the stack data so they're not unwound by sub-state pops.
    private _world!: wasm.World;
    private _board!: GameBoard;

    onPushed() {
        const canvas = document.getElementById("mainCanvas") as HTMLCanvasElement;
        this._world = new wasm.World();
        this._world.setTracer(new ConsoleTracer());
        this._board = new GameBoard(canvas, this._world, this.stack.boardListener(), this.stack.data);
        const update = this._updateWorld.bind(this);
        const getWorld = () => { return this._world; }
        const getBoard = () => { return this._board; }
        this.update(draft => {
            draft.build(LevelState.Data, getWorld, getBoard, update);
        });
    }

    onActivated() {
        this.update((draft) => {
            const focus = draft.build(Focus);
            focus.creature = {
                onEnter: (id) => this.update(draft => {
                    this._selectCreature(draft, id);
                }),
                onLeave: (id) => this.update(draft => {
                    this._unselectCreature(draft, id);
                }),
            };
            focus.part = {
                onEnter: ([cid, pid]) => this.update(draft => {
                    draft.build(Highlight).static.mutPartsFor(cid).inc(pid);
                }),
                onLeave: ([cid, pid]) => this.update(draft => {
                    draft.build(Highlight).static.mutPartsFor(cid).dec(pid);
                }),
            }
        });
    }

    onDeactivated() {
        this.update(draft => {
            draft.set(Highlight);
            draft.set(Focus);
        });
    }

    onPopped() {
        const data = this.stack.data.get(LevelState.Data)!;
        data.board.stop();
        data.world.free();
    }

    onTileEntered(hex: Hex) {
        const level = this.stack.data.get(LevelState.Data)!;
        let creature = level.creatureAt(hex);
        if (creature) {
            let id = creature.id;
            this.stack.data.get(Focus)?.creature.onEnter?.(id);
        }
    }

    onTileExited(hex: Hex) {
        const level = this.stack.data.get(LevelState.Data)!;
        let creature = level.creatureAt(hex);
        if (creature) {
            let id = creature.id;
            this.stack.data.get(Focus)?.creature.onLeave?.(id);
        }
    }

    private _updateWorld(newWorld: wasm.World) {
        this._world.free();
        this._world = newWorld;
        this._board.updateWorld(this._world);
        this.update(draft => {});
    }

    private _selectCreature(draft: Stack.Data, id: Id<wasm.Creature>) {
        const world = this.stack.data.get(LevelState.Data)!.world;
        const highlight = draft.build(Highlight);
        highlight.range = wasm.findBoundary(world.getCreatureRange(id));
        highlight.shade = world.shadeFrom(world.getCreatureHex(id)!, id);
        highlight.static.creatures.inc(id);
    }

    private _unselectCreature(draft: Stack.Data, id: Id<wasm.Creature>) {
        const highlight = draft.build(Highlight);
        highlight.range = [];
        highlight.shade = [];
        highlight.static.creatures.dec(id);
    }
}
export namespace LevelState {
    export class Data {
        [Stack.Datum] = true;
        [immerable] = true;

        floats: FloatText.ItemSet = new FloatText.ItemSet();
        constructor(
            private _getWorld: () => wasm.World,
            private _getBoard: () => GameBoard,
            public updateWorld: (newWorld: wasm.World) => void,
        ) {}

        get world(): wasm.World { return this._getWorld(); }
        get board(): GameBoard { return this._getBoard(); }

        creatureAt(hex: wasm.Hex): wasm.Creature | undefined {
            const tile = this.world.getTile(hex);
            if (!tile) { return undefined; }
            const id = tile.creature;
            if (id == undefined) { return undefined; }
            return this.world.getCreature(id);
        }
        
        makeFloat(event: Readonly<wasm.Event>): FloatText.Item | undefined {
            if (event.tags.includes("Normal")) {
                return undefined;
            }
            let cid = pathCreature(event.target);
            if (cid == undefined) { return undefined; }
            let pos = this.board.creatureCoords(cid)!;
            pos = new DOMPoint(pos.x, pos.y);  // clone
            let creature = this.world.getCreature(cid)!;

            let pid = pathPart(event.target);
            let part: wasm.Part | undefined;
            if (pid != undefined) {
                part = creature.parts.get(pid)!;
            }

            let data;
            if (data = event.data.ChangeHP) {
                const [text, color] = delta(data.delta);
                return {
                    pos,
                    text: `${part!.name}: ${text} HP`,
                    style: { color },
                };
            } else if (data = event.data.TagsSet) {
                let strs = data.tags.map(t => `+${t}`);
                return {
                    pos,
                    text: `${part!.name}: ${strs.join(", ")}`,
                };
            } else if (data = event.data.TagsCleared) {
                let strs = data.tags.map(t => `-${t}`);
                return {
                    pos,
                    text: `${part!.name}: ${strs.join(", ")}`,
                };
            } else if (data = event.data.ChangeAP) {
                const [text, color] = delta(data.delta);
                return {
                    pos,
                    text: `${text} AP`,
                    style: { color },
                };
            } else if (data = event.data.ChangeMP) {
                const [text, color] = delta(data.delta);
                return {
                    pos,
                    text: `${text} MP`,
                    style: { color },
                };
            } else if (data = event.data.Died) {
                return {pos, text: "Dead!"}
            } else if (data = event.data.FloatText) {
                return { pos, text: data.text };
            }
        }

        getIntents(): [Id<wasm.Creature>, wasm.Intent, DOMPointReadOnly][] {
            const intents: [Id<wasm.Creature>, wasm.Intent, DOMPointReadOnly][] = [];
            for (let creature of this.world.getCreatures()) {
                if (creature.dead || !creature.npc) { continue; }
                const intent = this.world.scaledIntent(creature.id);
                if (intent == undefined) { continue; }
                const point = this.board.creatureCoords(creature.id);
                if (!point) { continue; }
                intents.push([creature.id, intent, point]);
            }
            return intents;
        }
    }
}

class ConsoleTracer implements wasm.Tracer {
    resolveAction(action: string, events: wasm.Event[]) {
        console.log("ACTION:", action);
        for (let event of events) {
            console.log("==>", event);
        }
    }
    systemEvent(events: wasm.Event[]) {
        console.log("SYSTEM:", events[0]);
        for (let event of events.slice(1)) {
            console.log("==>", event);
        }
    }
}

class BufferTracer implements wasm.Tracer {
    private _buffer: (() => void)[] = [];
    constructor(private _wrapped: wasm.Tracer) {}
    resolveAction(action: string, events: wasm.Event[]) {
        this._buffer.push(() => this._wrapped.resolveAction(action, events));
    }
    systemEvent(events: wasm.Event[]) {
        this._buffer.push(() => this._wrapped.systemEvent(events));
    }
    runBuffer() {
        for (let thunk of this._buffer) {
            thunk();
        }
    }
}

function delta(value: number): [string, string] /* text, color */ {
    const sign = value < 0 ? "-" : "+";
    const color = value < 0 ? "#FF0000" : "#00FF00";
    return [`${sign}${Math.abs(value)}`, color]
}