import {World, Creature} from "../wasm";

declare module "../wasm" {
    interface World {
        readonly playerId: Id<Creature>;
        getTiles(): Array<[Hex, Tile]>;
        getTile(hex: Hex): Tile | undefined;
        getCreatureMap(): [Id<Creature>, Hex][];
        getCreature(id: Id<Creature>): Creature | undefined;
        getXCreature(id: Id<Creature>): XCreature | undefined;
        getCreatureHex(id: Id<Creature>): Hex | undefined;
        getCreatureRange(id: Id<Creature>): Hex[];
        checkSpendAP(id: Id<Creature>, ap: number): boolean;
        npcTurn(): Event[];

        _toFree: any[] | undefined;
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

World.prototype.getXCreature = function(id: Id<Creature>) {
    const val = this._getXCreature(id);
    if (val != undefined) {
        if (this._toFree == undefined) {
            this._toFree = [];
        }
        this._toFree.push(val);
    }
    return val;
}

const _oldWorldFree = World.prototype.free;
World.prototype.free = function() {
    if (this._toFree != undefined) {
        for (let obj of this._toFree) {
            if (obj.ptr != 0) {
                obj.free();
            }
        }
    }
    _oldWorldFree.bind(this)();
}

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