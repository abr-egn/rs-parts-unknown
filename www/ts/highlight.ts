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
    
    setPreview(preview: Readonly<Preview[]>) {
        this._stats = new Map();
        this._float = [];
        this._throb = [];
        for (let prev of preview) {
            this.addPreview(prev);
        }
    }

    addPreview(prev: Readonly<Preview>) {
        // TODO: show p.affects
        let act;
        if (act = prev.action.ToCreature) {
            let tc;
            if (tc = act.action.GainAP) {
                this._addDelta(act.id, "AP", tc.ap);
            } else if (tc = act.action.SpendAP) {
                this._addDelta(act.id, "AP", -tc.ap);
            } else if (tc = act.action.GainMP) {
                this._addDelta(act.id, "MP", tc.mp);
            } else if (tc = act.action.SpendMP) {
                this._addDelta(act.id, "MP", -tc.mp);
            }
        } else if (act = prev.action.MoveCreature) {
            this._throb.push(prev.action.MoveCreature.to);
        }
        /* TODO(hit preview)
        else if (act = prev.action.HitCreature) {
            throb.push(this._cache.creatureHex.get(p.action.HitCreature.id)!);
            const hex = window.game.world.getCreatureHex(act.id)!;
            this._float.push({
                text: `-${act.damage} HP`,
                pos: hexToPixel(hex),
                style: "#FF0000",
            });
        }
        */
    }

    private _addDelta(id: Id<wasm.Creature>, stat: Stat, delta: number) {
        let c = this.stats.get(id);
        if (!c) {
            c = {
                statDelta: new Map(),
                hpDelta: new Map(),
            };
            this.stats.set(id, c);
        }
        let oldDelta = c.statDelta.get(stat) || 0;
        c.statDelta.set(stat, oldDelta + delta);
    }
}

export type Stat = "AP" | "MP";

type StatMap = Map<Id<wasm.Creature>, StatPreview>;

export interface StatPreview {
    statDelta: Map<Stat, number>,
    hpDelta: Map<Id<wasm.Part>, number>,
}

export interface Preview {
    action: wasm.Action,
    affects: string[],
}

export namespace Preview {
    export function make(act: wasm.Action): Preview {
        return {
            action: act,
            affects: window.game.world.affectsAction(act),
        };
    }
}