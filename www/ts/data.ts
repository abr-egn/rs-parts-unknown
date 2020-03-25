interface Hex {
    x: number,
    y: number,
}

interface Tile {
    space: "Empty" | "Wall",
    creature: number | undefined,
}

interface Creature {
    hex: Hex,
    label: string,
}

interface Display {
    map: Array<[Hex, Tile]>,
    player_id: number,
    creatures: any,  // Map<number, Creature>
}