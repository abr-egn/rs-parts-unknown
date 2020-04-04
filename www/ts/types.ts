import {World} from "../wasm";

declare module "../wasm" {
    interface World {
        getTiles(): Array<[Hex, Tile]>;
    }
}
World.prototype.getTiles = function() { return this._getTiles() }

export interface Hex {
    x: number,
    y: number,
}

export interface Tile {
    space: Space,
    creature?: number,
}

export type Space = "Empty" | "Wall";