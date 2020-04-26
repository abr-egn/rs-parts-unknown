import {immerable} from "immer";

import * as wasm from "../wasm";
import {Id} from "../wasm";
import {Hex} from "../wasm";
import {Stack} from "./stack";

export class Highlight {
    [Stack.Datum] = true;
    [immerable] = true;

    throb: Hex[] = [];
    range: wasm.Boundary[] = [];
    creatures: CountMap<Id<wasm.Creature>> = new CountMap();
    parts: CountMap<Id<wasm.Part>> = new CountMap();
}

class CountMap<K> {
    [immerable] = true;

    private _data: Map<K, number> = new Map();

    inc(key: K) {
        let n = this._data.get(key) || 0;
        this._data.set(key, n+1);
    }
    dec(key: K) {
        let n = this._data.get(key) || 0;
        if (n == 1) {
            this._data.delete(key);
        } else {
            this._data.set(key, n-1);
        }
    }
    has(key: K): boolean {
        return Boolean(this._data.get(key));
    }
    all(): IterableIterator<K> {
        return this._data.keys();
    }
}