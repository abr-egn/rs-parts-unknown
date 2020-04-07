import {World, Behavior, _find_boundary} from "../wasm";

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
        pathTo(to: Hex): Hex[] | undefined;

        startPlay(card: Card): Behavior | undefined;
        npcTurn(): Event[];
        spendAP(id: Id<Creature>, ap: number): Event[];
        movePlayer(to: Hex): Event[];
    }
}

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

World.prototype.startPlay = World.prototype._startPlay;
World.prototype.npcTurn = World.prototype._npcTurn;
World.prototype.spendAP = World.prototype._spendAP;
World.prototype.movePlayer = World.prototype._movePlayer;

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
    data: {
        Nothing: {} | undefined,
        Failed: {
            action: any,
            reason: string,
        } | undefined,
        CreatureMoved: {
            id: Id<Creature>,
            from: Hex,
            to: Hex,
        } | undefined,
        SpentAP: {
            id: Id<Creature>,
            ap: number,
        }
    },
    tags: string[],
}

export function isFailure(events: Event[]): boolean {
    if (events.length < 1) {
        return false;
    }
    return events[0].data.Failed != undefined;
}

export interface Creature {
    id: Id<Creature>,
    parts: Map<Id<Part>, Part>,
    curAp: number,
    curMp: number,
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

/* Behavior */

declare module "../wasm" {
    interface Behavior {
        highlight(world: World, cursor: Hex): Hex[];
        targetValid(world: World, cursor: Hex): boolean;
        apply(world: World, target: Hex): Event[];
    }
}

Behavior.prototype.highlight = Behavior.prototype._highlight;
Behavior.prototype.targetValid = Behavior.prototype._targetValid;
Behavior.prototype.apply = Behavior.prototype._apply;

/* Boundary */

export interface Boundary {
    hex: Hex,
    sides: Direction[],
}

export type Direction = "XY" | "XZ" | "YZ" | "YX" | "ZX" | "ZY";

export const find_boundary: (shape: Hex[]) => Boundary[] = _find_boundary;