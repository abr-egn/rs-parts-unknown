import * as wasm from "../wasm";
import {Id} from "../wasm";
import {Hex} from "../wasm";
import {Stack} from "./stack";

export class Highlight {
    [Stack.Datum] = true;

    hexes: Hex[] = [];
    range: wasm.Boundary[] = [];
    creatures: CountMap<Id<wasm.Creature>> = newCount(new Map());
    parts: CountMap<Id<wasm.Part>> = newCount(new Map());
}

interface CountMap<K> extends Map<K, number> {
    inc(key: K): void;
    dec(key: K): void;
}

function newCount<K>(map: Map<K, number>): CountMap<K> {
    const out = map as CountMap<K>;
    out.inc = function(key: K) {
        let n = this.get(key) || 0;
        this.set(key, n+1);
    };
    out.dec = function(key: K) {
        let n = this.get(key) || 0;
        if (n == 1) {
            this.delete(key);
        } else {
            this.set(key, n-1);
        }
    }
    return out;
}