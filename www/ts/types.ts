export interface Hex {
    x: number,
    y: number,
}

export interface Tile {
    space: Space,
    creature: number,
}

export type Space = "Empty" | "Wall";