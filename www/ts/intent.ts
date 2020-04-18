import * as wasm from "../wasm";

import {Stack} from "./stack";

export class Intent {
    [Stack.Datum] = true;
    constructor(public npcs: [wasm.NPC, DOMPointReadOnly][]) {}
}