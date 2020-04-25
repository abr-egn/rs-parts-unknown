import * as wasm from "../wasm";
import {Id} from "../wasm";

import {Stack} from "./stack";

export class StatPreview {
    [Stack.Datum] = true;

    private _stats: StatMap = new Map();
    get stats(): Readonly<StatMap> { return this._stats; }

    setEvents(events: Readonly<wasm.Event[]>) {
        this._stats = new Map();
        for (let event of events) {
            this.addEvent(event);
        }
    }

    addEvent(event: Readonly<wasm.Event>) {
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

    private _addStatDelta(id: Id<wasm.Creature>, stat: Stat, delta: number) {
        let c = this._getStats(id);
        let oldDelta = c.statDelta.get(stat) || 0;
        c.statDelta.set(stat, oldDelta + delta);
    }

    private _addHpDelta(creatureId: Id<wasm.Creature>, partId: Id<wasm.Part>, delta: number) {
        let c = this._getStats(creatureId);
        let oldDelta = c.hpDelta.get(partId) || 0;
        c.hpDelta.set(partId, oldDelta + delta);
    }

    private _getStats(id: Id<wasm.Creature>): StatView {
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

export type Stat = "AP" | "MP";

type StatMap = Map<Id<wasm.Creature>, StatView>;

export interface StatView {
    statDelta: Map<Stat, number>,
    hpDelta: Map<Id<wasm.Part>, number>,
}