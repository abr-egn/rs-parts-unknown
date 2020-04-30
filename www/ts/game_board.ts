import {
    Creature, Hex, Id, Tile, World,
} from "../wasm";

import {
    Draw,
    hexToPixel, pixelToHex,
} from "./draw";
import {Highlight} from "./stack/highlight";
import {Preview} from "./stack/preview";
import {Stack} from "./stack";

export class GameBoard implements GameBoard.View {
    private readonly _draw: Draw;
    private _mouseHex?: Hex;
    private _tsMillis: DOMHighResTimeStamp;
    private _frameWaits: ((value: number) => void)[] = [];
    private _creaturePos: Map<Id<Creature>, DOMPointReadOnly> = new Map();
    private _cache!: WorldCache;
    constructor(
            canvas: HTMLCanvasElement,
            world: World,
            private readonly _listener: GameBoard.Listener,
            private readonly _data: Stack.DataView) {
        this._tsMillis = performance.now();
        this._draw = new Draw(canvas.getContext('2d')!);

        // TODO: move these up to Game
        canvas.addEventListener("mousedown", (event) => this._onMouseDown(event));
        canvas.addEventListener("mousemove", (event) => this._onMouseMove(event));
        window.requestAnimationFrame((ts) => this._frame(ts));

        this.updateWorld(world);
        // Render an initial frame to set up things like transform matrix.
        this._frame(performance.now());
    }

    updateWorld(world: World) {
        this._cache = new WorldCache(world);
        this._creaturePos.clear();
        for (let [id, hex] of world.getCreatureMap()) {
            this._creaturePos.set(id, hexToPixel(hex));
        }
    }

    creatureCoords(id: Id<Creature>): DOMPointReadOnly | undefined {
        const pos = this._creaturePos.get(id);
        if (!pos) { return undefined; }
        return this._draw.elementCoords(pos);
    }

    hexCoords(hex: Hex): DOMPointReadOnly {
        return this._draw.elementCoords(hexToPixel(hex));
    }

    async moveCreatureTo(id: number, dest: DOMPointReadOnly) {
        const MOVE_SPEED = 2.0;

        let start = this._creaturePos.get(id);
        if (start == undefined) { return; }
        let progress = 0.0;
        let prevTime = performance.now();
        while (progress < 1.0) {
            let time = await this._nextFrame();
            let delta = time - prevTime;
            prevTime = time;
            progress = progress + MOVE_SPEED*(delta/1000);
            let x = start.x + (dest.x - start.x)*progress;
            let y = start.y + (dest.y - start.y)*progress;
            this._creaturePos.set(id, new DOMPointReadOnly(x, y));
        }
    }

    private _nextFrame(): Promise<DOMHighResTimeStamp> {
        return new Promise((resolve) => this._frameWaits.push(resolve));
    }

    private _frame(tsMillis: DOMHighResTimeStamp) {
        this._tsMillis = tsMillis;

        this._draw.reset();

        for (let [hex, tile] of this._cache.tiles) {
            this._draw.tile(hex, tile);
        }

        for (let [id, pos] of this._creaturePos) {
            let text = "X";
            if (id == this._cache.playerId) {
                text = "P";
            }
            this._draw.creature(text, pos);
        }

        this._drawHighlight(this._data.get(Highlight));
        this._drawPreview(this._data.get(Preview));

        for (let resolve of this._frameWaits) {
            resolve(this._tsMillis);
        }
        this._frameWaits = [];

        window.requestAnimationFrame((ts) => this._frame(ts));
    }

    private _drawHighlight(hi?: Readonly<Highlight>) {
        if (!hi) { return; }
        for (let bound of hi.range) {
            this._draw.boundary(bound, "#808000");
        }
        for (let bound of hi.shade) {
            this._draw.shade(bound);
        }
        for (let hex of this._highlightHexes(hi.static)) {
            this._draw.highlightHex(hex);
        }
        for (let hex of this._highlightHexes(hi.throb)) {
            this._draw.throb(hex, this._tsMillis);
        }
    }

    private _highlightHexes(tracker: Readonly<Highlight.Tracker>): Hex[] {
        const all: Map<string, Hex> = new Map();
        for (let id of tracker.creatures.all()) {
            const hex = this._cache.creatureHex.get(id);
            if (hex) {
                all.set(JSON.stringify(hex), hex);
            }
        }
        for (let entry of tracker.parts) {
            const [cid, parts] = entry;
            const hex = this._cache.creatureHex.get(cid);
            if (!hex) { continue; }
            if (parts.all().length > 0) {
                all.set(JSON.stringify(hex), hex);
            }
        }
        return Array.from(all.values());
    }

    private _drawPreview(prev: Readonly<Preview> | undefined) {
        if (!prev) { return; }
        for (let hex of prev.throb) {
            this._draw.throb(hex, this._tsMillis);
        }
    }

    private _onMouseDown(event: MouseEvent) {
        event.preventDefault();
        this._draw.focus();
        const point = this._draw.mouseCoords(event);
        this._listener.onTileClicked(pixelToHex(point));
    }

    private _onMouseMove(event: MouseEvent) {
        const hex = pixelToHex(this._draw.mouseCoords(event));
        if (this._cache.getTile(hex) == undefined) { return; }
        if (hex.x != this._mouseHex?.x || hex.y != this._mouseHex?.y) {
            if (this._mouseHex) {
                this._listener.onTileExited(this._mouseHex);
            }
            this._listener.onTileEntered(hex);
            this._mouseHex = hex;
        }
    }
}
export namespace GameBoard {
    export interface Listener {
        onTileClicked(hex: Hex): void,
        onTileEntered(hex: Hex): void,
        onTileExited(hex: Hex): void,
    }
    
    type Constructor = new (...args: any[]) => any;
    
    export interface DataQuery {
        get<T extends Constructor>(key: T): Readonly<InstanceType<T>> | undefined;
    }

    export interface View {
        hexCoords(hex: Hex): DOMPointReadOnly;
        creatureCoords(id: Id<Creature>): DOMPointReadOnly | undefined;
    }

    export interface Motion {
        moveCreatureTo(id: number, dest: DOMPointReadOnly): Promise<void>;
    }
}

class WorldCache {
    tiles: [Hex, Tile][];
    creatures: Map<Id<Creature>, Creature>= new Map();
    creatureHex: Map<Id<Creature>, Hex> = new Map();
    playerId: Id<Creature>;

    private _tileMap: Map<string, Tile> = new Map();

    constructor(world: World) {
        this.tiles = world.getTiles();
        for (let [hex, tile] of this.tiles) {
            this._tileMap.set(JSON.stringify(hex), tile);
        }
        for (let [id, hex] of world.getCreatureMap()) {
            this.creatureHex.set(id, hex);
            this.creatures.set(id, world.getCreature(id)!);
        }
        this.playerId = world.playerId;
    }

    getTile(hex: Hex): Tile | undefined {
        return this._tileMap.get(JSON.stringify(hex));
    }
}
