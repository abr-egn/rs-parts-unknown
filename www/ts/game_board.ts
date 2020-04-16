import {
    Creature, Event, Hex, Id, Tile, World,
} from "../wasm";
import {
    Draw, FloatText,
    hexToPixel, pixelToHex,
} from "./draw";
import * as states from "./states";

export class GameBoard {
    private readonly _draw: Draw;
    private _mouseHex?: Hex;
    private _tsMillis: DOMHighResTimeStamp;
    private _frameWaits: ((value: number) => void)[] = [];
    private _creaturePos: Map<Id<Creature>, DOMPointReadOnly> = new Map();
    private _cache!: WorldCache;
    private _floatText: Set<FloatText> = new Set();
    constructor(
            canvas: HTMLCanvasElement,
            world: World,
            private readonly _listener: GameBoard.Listener,
            private readonly _data: GameBoard.DataQuery) {
        canvas.width = window.innerWidth;
        canvas.height = window.innerHeight;
        window.onresize = () => {
            canvas.width = window.innerWidth;
            canvas.height = window.innerHeight;
        };
        this.updateWorld(world);
        this._tsMillis = performance.now();
        this._draw = new Draw(canvas.getContext('2d')!);
        canvas.addEventListener("mousedown", (event) => this._onMouseDown(event));
        canvas.addEventListener("mousemove", (event) => this._onMouseMove(event));
        window.requestAnimationFrame((ts) => this._frame(ts));
    }

    updateWorld(world: World) {
        this._cache = new WorldCache(world);
        this._creaturePos.clear();
        for (let [id, hex] of world.getCreatureMap()) {
            this._creaturePos.set(id, hexToPixel(hex));
        }
    }

    async animateEvents(events: Event[]) {
        for (let event of events) {
            let data;
            // TODO: update UI for AP/MP changes
            if (data = event.CreatureMoved) {
                await this._moveCreatureTo(data.id, hexToPixel(data.to))
            }
            if (data = event.OnCreature?.event.OnPart?.event.ChangeHP) {
                const FLOAT_SPEED = 10.0;

                const creature_id = event.OnCreature.id;
                const creature = this._cache.creatures.get(creature_id)!;
                const part = creature.parts.get(event.OnCreature.event.OnPart.id)!;
                let point = this._creaturePos.get(creature_id)!;
                const sign = data.delta < 0 ? "-" : "+";
                let float: FloatText = {
                    pos: new DOMPoint(point.x, point.y),
                    text: `${part.name}: ${sign}${Math.abs(data.delta)} HP`,
                    style: "#FF0000",
                };
                this._floatText.add(float);
                const start = this._tsMillis;
                let now = this._tsMillis;
                while (now - start < 2000) {
                    const tmp = await this._nextFrame();
                    const delta = tmp - now;
                    now = tmp;
                    float.pos.y -= FLOAT_SPEED*(delta/1000);
                }
                this._floatText.delete(float);
            }
        }
    }

    private _nextFrame(): Promise<DOMHighResTimeStamp> {
        return new Promise((resolve) => this._frameWaits.push(resolve));
    }

    private async _moveCreatureTo(id: number, dest: DOMPointReadOnly) {
        const MOVE_SPEED = 1.0;

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

        const selected = this._data.get(states.Base.UI)?.selected || [];
        for (let [id, bounds] of selected) {
            for (let bound of bounds) {
                this._draw.boundary(bound);
            }
            const hex = this._cache.creatureHex.get(id);
            if (hex) {
                this._draw.focusedHex(hex);
            }
        }

        this._drawHighlight(this._data.get(states.Highlight));

        for (let float of this._floatText) {
            this._draw.floatText(float);
        }

        for (let resolve of this._frameWaits) {
            resolve(this._tsMillis);
        }
        this._frameWaits = [];

        window.requestAnimationFrame((ts) => this._frame(ts));
    }

    private _drawHighlight(hi?: Readonly<states.Highlight>) {
        if (!hi) { return; }
        for (let bound of hi.range) {
            this._draw.boundary(bound);
        }
        for (let hex of hi.hexes) {
            this._draw.focusedHex(hex);
        }
        for (let hex of hi.throb) {
            this._draw.throb(hex, this._tsMillis);
        }
        for (let float of hi.float) {
            this._draw.floatText(float);
        }
    }

    private _onMouseDown(event: MouseEvent) {
        event.preventDefault();
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
