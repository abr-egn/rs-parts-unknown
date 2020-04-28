import {immerable} from "immer";

import * as wasm from "../../wasm";
import {Hex, Id} from "../../wasm";
import {Stack} from "../stack";

import {FloatText} from "../../tsx/float";

export class Preview {
    [Stack.Datum] = true;
    [immerable] = true;

    private _stats: StatMap = new Map();
    private _float: FloatText.ItemSet = new FloatText.ItemSet();
    private _throb: Hex[] = [];

    get float(): Readonly<FloatText.ItemId[]> { return this._float.all }
    get throb(): Readonly<Hex[]> { return this._throb; }
    get stats(): Readonly<StatMap> { return this._stats; }
    set stats(v) {}  // Work around immer bug: https://github.com/immerjs/immer/pull/558

    setEvents(events: Readonly<wasm.Event[]>) {
        this._stats = new Map();
        this._float = new FloatText.ItemSet();
        this._throb = [];
        // TASK: when there are multiple floats, stagger the positions
        for (let event of events) {
            this.addStats(event);
            const float = Preview.float(event);
            if (float) {
                this._float.add(float);
            }
            this._addOther(event);
        }
    }

    addStats(event: Readonly<wasm.Event>) {
        let ev;
        if (ev = event.OnCreature) {
            let oc;
            if (oc = ev.event.ChangeAP) {
                this._addStatDelta(ev.id, "AP", oc.delta);
            } else if (oc = ev.event.ChangeMP) {
                this._addStatDelta(ev.id, "MP", oc.delta);
            } else if (oc = ev.event.OnPart) {
                let op;
                if (op = oc.event.ChangeHP) {
                    this._addHpDelta(ev.id, oc.id, op.delta);
                }
            }
        }
    }

    private _addOther(event: Readonly<wasm.Event>) {
        let ev;
        if (ev = event.CreatureMoved) {
            this._throb.push(ev.from, ev.to);
        }
    }

    private _addStatDelta(id: Id<wasm.Creature>, stat: Preview.Stat, delta: number) {
        let c = this._getStats(id);
        let oldDelta = c.statDelta.get(stat) || 0;
        c.statDelta.set(stat, oldDelta + delta);
    }

    private _addHpDelta(creatureId: Id<wasm.Creature>, partId: Id<wasm.Part>, delta: number) {
        let c = this._getStats(creatureId);
        let oldDelta = c.hpDelta.get(partId) || 0;
        c.hpDelta.set(partId, oldDelta + delta);
    }

    private _getStats(id: Id<wasm.Creature>): Preview.Stats {
        let c = this._stats.get(id);
        if (!c) {
            c = {
                statDelta: new Map(),
                hpDelta: new Map(),
            };
            this._stats.set(id, c);
        }
        return c;
    }
}
export namespace Preview {
    export type Stat = "AP" | "MP";
    export interface Stats {
        statDelta: Map<Stat, number>,
        hpDelta: Map<Id<wasm.Part>, number>,
    }
    export function float(event: Readonly<wasm.Event>): FloatText.Item | undefined {
        let ev;
        if (ev = event.OnCreature) {
            let pos = window.game.board.creatureCoords(ev.id)!;
            pos = new DOMPoint(pos.x, pos.y);  // clone
            let creature = window.game.world.getCreature(ev.id)!;
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
            let pos = window.game.board.creatureCoords(ev.on);
            if (pos) {
                return {
                    pos, text: ev.text, style: {}
                };
            }
        }
    }
}

type StatMap = Map<Id<wasm.Creature>, Preview.Stats>;

function delta(value: number): [string, string] /* text, color */ {
    const sign = value < 0 ? "-" : "+";
    const color = value < 0 ? "#FF0000" : "#00FF00";
    return [`${sign}${Math.abs(value)}`, color]
}