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
            let oc;
            if (oc = ev.event.OnPart) {
                let op;
                if (op = oc.event.ChangeHP) {
                    return hpFloat(ev.id, oc.id, op.delta);
                }
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

function hpFloat(creatureId: Id<wasm.Creature>, partId: Id<wasm.Part>, delta: number): FloatText.Item | undefined {
    let point = window.game.board.creatureCoords(creatureId);
    if (!point) { return; }
    let creature = window.game.world.getCreature(creatureId);
    if (!creature) { return; }
    let part = creature.parts.get(partId);
    if (!part) { return; }
    const sign = delta < 0 ? "-" : "+";
    const color = delta < 0 ? "#FF0000" : "#00FF00";
    return {
        pos: new DOMPoint(point.x, point.y),
        text: `${part.name}: ${sign}${Math.abs(delta)} HP`,
        style: { color },
    };
}