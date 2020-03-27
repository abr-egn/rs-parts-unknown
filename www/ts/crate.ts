import {PartsUnknown} from "../wasm";

export class Crate {
    constructor(
        public readonly wasm: typeof import("../wasm"),
        public readonly backend: PartsUnknown,
    ) {}
}