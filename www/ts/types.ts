import {World} from "../wasm";

/* World */

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

        startPlay(card: Card): Behavior | undefined;
        npcTurn(): Event[];
    }
}

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
World.prototype.getCreature = World.prototype._getCreature;
World.prototype.startPlay = World.prototype._startPlay;

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

export interface Creature {
    id: Id<Creature>,
    kind: {
        Player: Player | undefined,
        NPC: NPC | undefined,
    },
    parts: Map<Id<Part>, Part>,
    curAp: number,
}

export interface Player {}
export interface NPC {
    move_range: number,
    attack_range: number,
}

export interface Part {
    id: Id<Part>,
    creatureId: Id<Creature>,
    cards: Map<Id<Card>, Card>,
    ap: number,
}

export interface Card {
    id: Id<Card>,
    partId: Id<Part>,
    creatureId: Id<Creature>,
    name: string,
    apCost: number,
}

// TODO: Behavior typing