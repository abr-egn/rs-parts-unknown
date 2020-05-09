import {immerable} from "immer";

import {Id, Hex} from "../../wasm";
import {FloatText} from "../../tsx/float";
import * as wasm from "../../wasm";
import {GameBoard} from "../game_board";
import {Stack, State} from "../stack";
import {Focus} from "../stack/focus";
import {Highlight} from "../stack/highlight";

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
            this.update(draft => this._selectCreature(draft, id));
        }
    }

    onTileExited(hex: Hex) {
        const level = this.stack.data.get(LevelState.Data)!;
        let creature = level.creatureAt(hex);
        if (creature) {
            let id = creature.id;
            this.update(draft => this._unselectCreature(draft, id));
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
            let ev;
            if (ev = event.OnCreature) {
                let pos = this.board.creatureCoords(ev.id)!;
                pos = new DOMPoint(pos.x, pos.y);  // clone
                let creature = this.world.getCreature(ev.id)!;
                let oc;
                if (oc = ev.event.OnPart) {
                    let part = creature.parts.get(oc.id)!;
                    let op;
                    if (op = oc.event.ChangeHP) {
                        const [text, color] = delta(op.delta);
                        return {
                            pos,
                            text: `${part.name}: ${text} HP`,
                            style: { color },
                        };
                    } else if (op = oc.event.TagsSet) {
                        let strs = op.tags.map(t => `+${t}`);
                        return {
                            pos,
                            text: `${part.name}: ${strs.join(", ")}`,
                        };
                    } else if (op = oc.event.TagsCleared) {
                        let strs = op.tags.map(t => `-${t}`);
                        return {
                            pos,
                            text: `${part.name}: ${strs.join(", ")}`,
                        };
                    }
                } else if (oc = ev.event.ChangeAP) {
                    const [text, color] = delta(oc.delta);
                    return {
                        pos,
                        text: `${text} AP`,
                        style: { color },
                    };
                } else if (oc = ev.event.ChangeMP) {
                    const [text, color] = delta(oc.delta);
                    return {
                        pos,
                        text: `${text} MP`,
                        style: { color },
                    };
                } else if (oc = ev.event.Died) {
                    return {pos, text: "Dead!"}
                }
            } else if (ev = event.FloatText) {
                let pos = this.board.creatureCoords(ev.on);
                if (pos) {
                    return {
                        pos, text: ev.text, style: {}
                    };
                }
            }
        }

        getIntents(): [wasm.Creature, DOMPointReadOnly][] {
            const intents: [wasm.Creature, DOMPointReadOnly][] = [];
            for (let creature of this.world.getCreatures()) {
                if (creature.dead || !creature.npc) { continue; }
                const point = this.board.creatureCoords(creature.id);
                if (!point) { continue; }
                intents.push([creature, point]);
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