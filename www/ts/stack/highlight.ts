import {immerable} from "immer";

import * as wasm from "../../wasm";
import {Id} from "../../wasm";
import {Hex} from "../../wasm";
import {Stack} from "../stack";
import { Focus } from "./focus";

export class Highlight {
    [Stack.Datum] = true;
    [immerable] = true;

    throb: FocusTracker = new FocusTracker();
    range: wasm.Boundary[] = [];

    creatures: CountMap<Id<wasm.Creature>> = new CountMap();
    parts: Map<Id<wasm.Creature>, CountMap<Id<wasm.Part>>> = new Map();

    mutPartsFor(cid: Id<wasm.Creature>): CountMap<Id<wasm.Part>> {
        let out = this.parts.get(cid);
        if (!out) {
            out = new CountMap();
            this.parts.set(cid, out);
        }
        return out;
    }
}

class FocusTracker {
    [immerable] = true;

    creatures: CountMap<Id<wasm.Creature>> = new CountMap();
    parts: Map<Id<wasm.Creature>, CountMap<Id<wasm.Part>>> = new Map();

    mutPartsFor(cid: Id<wasm.Creature>): CountMap<Id<wasm.Part>> {
        let out = this.parts.get(cid);
        if (!out) {
            out = new CountMap();
            this.parts.set(cid, out);
        }
        return out;
    }

    clear() {
        this.creatures.clear();
        this.parts.clear();
    }
}

class CountMap<K> {
    [immerable] = true;

    private _data: Map<K, number> = new Map();

    inc(key: K) {
        let n = this._data.get(key);
        this._data.set(key, (n || 0)+1);
    }
    dec(key: K) {
        let n = this._data.get(key) || 0;
        if (n <= 1) {
            this._data.delete(key);
        } else {
            this._data.set(key, n-1);
        }
    }
    clear() {
        this._data.clear();
    }
    has(key: K): boolean {
        return Boolean(this._data.get(key));
    }
    all(): K[] {
        const out = [];
        for (let entry of this._data.entries()) {
            const [k, v] = entry;
            if (v > 0) { out.push(k); }
        }
        return out;
    }
}