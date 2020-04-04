import {World} from "../wasm";

declare module "../wasm" {
    interface World {
        readonly playerId: Id<Creature>;
        getTiles(): Array<[Hex, Tile]>;
        getTile(hex: Hex): Tile | undefined;
        getCreatureMap(): [Id<Creature>, Hex][];
        getCreature(id: Id<Creature>): Creature | undefined;
        getCreatureHex(id: Id<Creature>): Hex | undefined;
        getCreatureRange(id: Id<Creature>): Hex[];
        checkSpendAP(id: Id<Creature>, ap: number): boolean;
        npcTurn(): Event[];
    }
}
Object.defineProperty(World.prototype, "playerId", {
    get: function() { return this._playerId; }
})
World.prototype.getTiles = World.prototype._getTiles;
World.prototype.getTile = World.prototype._getTile;
World.prototype.getCreatureMap = World.prototype._getCreatureMap;
World.prototype.getCreature = World.prototype._getCreature;
World.prototype.getCreatureHex = World.prototype._getCreatureHex;
World.prototype.getCreatureRange = World.prototype._getCreatureRange;
World.prototype.checkSpendAP = World.prototype._checkSpendAP;
World.prototype.npcTurn = World.prototype._npcTurn;

export interface Hex {
    x: number,
    y: number,
}

export interface Tile {
    space: Space,
    creature?: number,
}

export type Space = "Empty" | "Wall";

export interface Id<_> {
    value: number,
}