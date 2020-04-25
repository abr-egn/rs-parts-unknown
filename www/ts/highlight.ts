import * as wasm from "../wasm";
import {Hex} from "../wasm";
import {Stack} from "./stack";

export class Highlight {
    [Stack.Datum] = true;

    hexes: Hex[] = [];
    range: wasm.Boundary[] = [];
}