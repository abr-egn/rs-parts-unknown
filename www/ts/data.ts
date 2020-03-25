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

export interface Display {
    map: Array<[Hex, Tile]>,
    player_id: number,
    creatures: Map<number, Creature>,
}