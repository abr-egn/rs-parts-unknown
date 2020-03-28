export interface Hex {
    x: number,
    y: number,
}

export interface Tile {
    space: "Empty" | "Wall",
    creature?: number,
}

export interface Creature {
    hex: Hex,
    label: string,
}

export interface World {
    map: [Hex, Tile][],
    player_id: number,
    creatures: Map<number, Creature>,
}

export interface Meta {
    data: GameEvent,
    tags: string[],
}

export type GameEvent = EvCreatureMoved | EvFailed;

export interface EvCreatureMoved {
    CreatureMoved: {
        id: number,
        path: Hex[],
    }
}

export interface EvFailed {
    Failed: {
        action: any,
        reason: string,
    }
}