import * as wasm from "../wasm";
import {Hex, Id} from "../wasm";
import {Stack} from "./stack";

import {FloatText} from "../tsx/float";

export class Preview {
    [Stack.Datum] = true;

    private _stats: StatMap = new Map();
    private _float: FloatText.ItemSet = new FloatText.ItemSet();
    private _throb: Hex[] = [];

    get float(): Readonly<FloatText.ItemId[]> { return this._float.all }
    get throb(): Readonly<Hex[]> { return this._throb; }
    get stats(): Readonly<StatMap> { return this._stats; }

    setEvents(events: Readonly<wasm.Event[]>) {
        this._stats = new Map();
        this._float = new FloatText.ItemSet();
        this._throb = [];
        for (let event of events) {
            this.addEvent(event);
        }
    }

    addEvent(event: Readonly<wasm.Event>, statOnly?: boolean) {
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
                    if (!statOnly) {
                        this._float.add(window.game.board.hpFloat(ev.id, oc.id, op.delta, true));
                    }
                }
            }
        } else if (ev = event.CreatureMoved) {
            if (!statOnly) {
                this._throb.push(ev.from, ev.to);
            }
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
        let c = this.stats.get(id);
        if (!c) {
            c = {
                statDelta: new Map(),
                hpDelta: new Map(),
            };
            this.stats.set(id, c);
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
}

type StatMap = Map<Id<wasm.Creature>, Preview.Stats>;