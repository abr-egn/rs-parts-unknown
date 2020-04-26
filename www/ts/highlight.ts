import * as wasm from "../wasm";
import {Id} from "../wasm";
import {Hex} from "../wasm";
import {Stack} from "./stack";

export class Highlight {
    [Stack.Datum] = true;

    hexes: Hex[] = [];
    range: wasm.Boundary[] = [];
    creatures: Set<Id<wasm.Creature>> = new Set();
    parts: Set<Id<wasm.Part>> = new Set();
}