import * as wasm from "../wasm";
import {Hex, Id} from "../wasm";
import {FloatText} from "./draw";
import {Stack} from "./stack";

export class Highlight {
    [Stack.Datum] = true;

    hexes: Hex[] = [];
    range: wasm.Boundary[] = [];

    private _stats: StatMap = new Map();
    private _float: FloatText[] = [];
    private _throb: Hex[] = [];

    get stats(): Readonly<StatMap> { return this._stats; }
    get float(): Readonly<FloatText[]> { return this._float; }
    get throb(): Readonly<Hex[]> { return this._throb; }
    
    setEvents(events: Readonly<wasm.Event[]>) {
        this._stats = new Map();
        this._float = [];
        this._throb = [];
        for (let event of events) {
            this.addEvents(event);
        }
    }

    addEvents(event: Readonly<wasm.Event>) {
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
                    this._float.push(window.game.board.hpFloat(ev.id, oc.id, op.delta));
                }
            }
        } else if (ev = event.CreatureMoved) {
            this._throb.push(ev.from, ev.to);
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

    private _getStats(id: Id<wasm.Creature>): StatPreview {
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

type StatMap = Map<Id<wasm.Creature>, StatPreview>;

export interface StatPreview {
    statDelta: Map<Stat, number>,
    hpDelta: Map<Id<wasm.Part>, number>,
}