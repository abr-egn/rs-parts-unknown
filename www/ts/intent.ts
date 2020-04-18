import * as wasm from "../wasm";

export class Intent {
    constructor(public npcs: [wasm.NPC, DOMPointReadOnly][]) {}
}