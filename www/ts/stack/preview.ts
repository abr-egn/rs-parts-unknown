import {immerable} from "immer";

import * as wasm from "../../wasm";
import {Hex, Id} from "../../wasm";

import {pathCreature, pathPart} from "../extra";
import {Stack} from "../stack";
import {LevelState} from "../states/level";

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

    setEvents(level: Readonly<LevelState.Data>, events: Readonly<wasm.Event[]>) {
        this._stats = new Map();
        this._float = new FloatText.ItemSet();
        this._throb = [];
        // TASK: when there are multiple floats, stagger the positions
        for (let event of events) {
            this.addStats(event);
            const float = level.makeFloat(event);
            if (float) {
                this._float.add(float);
            }
            this._addOther(event);
        }
    }

    addStats(event: Readonly<wasm.Event>) {
        let id = pathCreature(event.target);
        let data;
        if (data = event.data.ChangeAP) {
            this._addStatDelta(id!, "AP", data.delta);
        } else if (data = event.data.ChangeMP) {
            this._addStatDelta(id!, "MP", data.delta);
        } else if (data = event.data.ChangeHP) {
            let pid = pathPart(event.target)!;
            this._addHpDelta(id!, pid, data.delta);
        }
    }

    private _addOther(event: Readonly<wasm.Event>) {
        let data;
        if (data = event.data.Moved) {
            this._throb.push(data.from, data.to);
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
}

type StatMap = Map<Id<wasm.Creature>, Preview.Stats>;