export class Crate {
    constructor(public readonly wasm: typeof import("../wasm")) {}
}