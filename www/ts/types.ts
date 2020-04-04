import {World, Creature, XPlayer, XCreature} from "../wasm";

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

    interface XCreature {
        readonly player: XPlayer | undefined;

        _toFree: any[] | undefined;
    }
}

World.prototype.free = wrapFree(World.prototype.free);
Object.defineProperty(World.prototype, "playerId", {
    get: function() { return this._playerId; }
});
World.prototype.getTiles = World.prototype._getTiles;
World.prototype.getTile = World.prototype._getTile;
World.prototype.getCreatureMap = World.prototype._getCreatureMap;
World.prototype.getCreature = World.prototype._getCreature;
World.prototype.getCreatureHex = World.prototype._getCreatureHex;
World.prototype.getCreatureRange = World.prototype._getCreatureRange;
World.prototype.checkSpendAP = World.prototype._checkSpendAP;
World.prototype.npcTurn = World.prototype._npcTurn;
World.prototype.getXCreature = wrapGet(World.prototype._getXCreature);

XCreature.prototype.free = wrapFree(XCreature.prototype.free);
Object.defineProperty(XCreature.prototype, "player", {
    get: function() {
        return wrapGet(XCreature.prototype._player).bind(this)();
    }
});

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

interface FreeTracker {
    _toFree: any[] | undefined;
}

function wrapFree<B extends FreeTracker>(oldFree: () => void): (this: B) => void {
    return function() {
        if (this._toFree != undefined) {
            for (let obj of this._toFree) {
                if (obj.ptr != 0) {
                    obj.free();
                }
            }
        }
        oldFree.bind(this)();
    };
}

function wrapGet<
    C extends FreeTracker,
    T extends (this: C, ...args: any[]) => any,
>(inner: T): (this: C, ...args: Parameters<T>) => ReturnType<T> {
    return function(this: C, ...args: Parameters<T>): ReturnType<T> {
        const val = inner.bind(this)(...args);
        if (val != undefined) {
            if (this._toFree == undefined) {
                this._toFree = [];
            }
            this._toFree.push(val);
        }
        return val;
    }
}