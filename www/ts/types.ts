import {World, Creature, Card} from "../wasm";

/* World */

declare module "../wasm" {
    interface World {
        readonly playerId: Id<Creature>;
        getTiles(): Array<[Hex, Tile]>;
        getTile(hex: Hex): Tile | undefined;
        getCreatureMap(): [Id<Creature>, Hex][];
        //getCreature(id: Id<Creature>): Creature | undefined;
        withCreature<T>(id: Id<Creature>, f: (creature?: Creature) => T): T;
        getSCreature(id: Id<Creature>): SCreature | undefined;
        getCreatureHex(id: Id<Creature>): Hex | undefined;
        getCreatureRange(id: Id<Creature>): Hex[];
        checkSpendAP(id: Id<Creature>, ap: number): boolean;

        npcTurn(): Event[];
    }
}

World.prototype.free = wrapFree(World.prototype.free);
Object.defineProperty(World.prototype, "playerId", {
    get: function() { return this._playerId; }
});
World.prototype.getTiles = World.prototype._getTiles;
World.prototype.getTile = World.prototype._getTile;
World.prototype.getCreatureMap = World.prototype._getCreatureMap;
World.prototype.getCreatureHex = World.prototype._getCreatureHex;
World.prototype.getCreatureRange = World.prototype._getCreatureRange;
World.prototype.checkSpendAP = World.prototype._checkSpendAP;
World.prototype.npcTurn = World.prototype._npcTurn;
World.prototype.withCreature = function<T>(id: Id<Creature>, f: (creature?: Creature) => T): T {
    let creature = this._getCreature(id);
    let out = f(creature);
    creature?.free();
    return out;
};
//World.prototype.getCreature = wrapGet(World.prototype._getCreature);

export interface Hex {
    x: number,
    y: number,
}

export interface Tile {
    space: Space,
    creature?: number,
}

export type Space = "Empty" | "Wall";

export type Id<_> = number;

export interface Event {
    data: any,
    tags: string[],
}

export interface SCreature {
    id: Id<Creature>,
    data: {
        kind: {
            Player: SPlayer | undefined,
            NPC: SNPC | undefined,
        },
        parts: Map<Id<SPart>, SPart>,
        cur_ap: number,
    },
}

export interface SPlayer {}
export interface SNPC {
    move_range: number,
    attack_range: number,
}

export interface SPart {
    cards: Map<any, any>,
    ap: number,
}

export interface SCard {
    name: string,
    ap_cost: number,
}

/* Creature */

declare module "../wasm" {
    interface Creature {
        getPlayer(): Player | undefined;
        getNPC(): NPC | undefined;
        getCards(): Card[];

        _toFree: any[] | undefined;
    }
}

Creature.prototype.free = wrapFree(Creature.prototype.free);
Creature.prototype.getPlayer = wrapGet(Creature.prototype._player);
Creature.prototype.getNPC = Creature.prototype._npc;
Creature.prototype.getCards = function() {
    const values = this._cards();
    if (this._toFree == undefined) {
        this._toFree = [];
    }
    this._toFree.push(...values);
    return values;
}

export interface NPC {
    move_range: number,
    attack_range: number,
}

/* Card */

declare module "../wasm" {
    interface Card {
        startPlay(world: World, source: Id<Creature>): Behavior;
    }
}

Card.prototype.startPlay = wrapGet(Card.prototype._startPlay);

/* Child object tracking machinery */

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