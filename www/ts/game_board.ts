import {
    Creature, Event, Hex, Id, Part, Tile, World,
} from "../wasm";

import {
    Draw, FloatText,
    hexToPixel, pixelToHex,
} from "./draw";
import {Highlight} from "./highlight";
import {Stack} from "./stack";
import * as states from "./states";

export class GameBoard implements GameBoard.View {
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
            private readonly _data: Stack.DataView) {
        canvas.width = window.innerWidth;
        canvas.height = window.innerHeight;
        window.onresize = () => {
            canvas.width = window.innerWidth;
            canvas.height = window.innerHeight;
        };

        this._tsMillis = performance.now();
        this._draw = new Draw(canvas.getContext('2d')!);

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

    async animateEvents(events: Event[], preview: (ev: Event) => void) {
        for (let event of events) {
            preview(event);
            let data;
            if (data = event.CreatureMoved) {
                await this._moveCreatureTo(data.id, hexToPixel(data.to))
            } else if (data = event.OnCreature) {
                let ce;
                if (ce = data.event.OnPart) {
                    let pe;
                    if (pe = ce.event.ChangeHP) {
                        const FLOAT_SPEED = 10.0;

                        const float = this.hpFloat(data.id, ce.id, pe.delta);
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
        }
    }

    hpFloat(creatureId: Id<Creature>, partId: Id<Part>, delta: number): FloatText {
        const creature = this._cache.creatures.get(creatureId)!;
        const part = creature.parts.get(partId)!;
        let point = this._creaturePos.get(creatureId)!;
        const sign = delta < 0 ? "-" : "+";
        return {
            pos: new DOMPoint(point.x, point.y),
            text: `${part.name}: ${sign}${Math.abs(delta)} HP`,
            style: "#FF0000",
        };
    }

    creatureCoords(id: Id<Creature>): DOMPointReadOnly | undefined {
        const pos = this._creaturePos.get(id);
        if (!pos) { return undefined; }
        return this._draw.elementCoords(pos);
    }

    hexCoords(hex: Hex): DOMPointReadOnly {
        return this._draw.elementCoords(hexToPixel(hex));
    }

    private _nextFrame(): Promise<DOMHighResTimeStamp> {
        return new Promise((resolve) => this._frameWaits.push(resolve));
    }

    private async _moveCreatureTo(id: number, dest: DOMPointReadOnly) {
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

        if (!this._data.get(Stack.Active)?.is(states.Update)) {
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
        }

        this._drawHighlight(this._data.get(Highlight));

        for (let float of this._floatText) {
            this._draw.floatText(float);
        }

        for (let resolve of this._frameWaits) {
            resolve(this._tsMillis);
        }
        this._frameWaits = [];

        window.requestAnimationFrame((ts) => this._frame(ts));
    }

    private _drawHighlight(hi?: Readonly<Highlight>) {
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
        hpFloat(creatureId: Id<Creature>, partId: Id<Part>, delta: number): FloatText;    
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
