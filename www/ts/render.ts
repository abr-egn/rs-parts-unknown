import {
    Boundary, Creature, Event, Hex, Id, Tile, World,
} from "../wasm";
import * as states from "./states";

export class Render {
    private readonly _ctx: CanvasRenderingContext2D;
    private _mouseHex?: Hex;
    private _tsMillis: DOMHighResTimeStamp;
    private _frameWaits: ((value: number) => void)[] = [];
    private _creaturePos: Map<Id<Creature>, DOMPointReadOnly> = new Map();
    private _cache!: WorldCache;
    private _floatText: Set<FloatText> = new Set();
    constructor(
            private readonly _canvas: HTMLCanvasElement,
            world: World,
            private readonly _listener: Listener,
            private readonly _data: DataQuery) {
        this._canvas.width = window.innerWidth;
        this._canvas.height = window.innerHeight;
        window.onresize = () => {
            this._canvas.width = window.innerWidth;
            this._canvas.height = window.innerHeight;
        };
        this.updateWorld(world);
        this._tsMillis = performance.now();
        this._ctx = this._canvas.getContext('2d')!;
        this._canvas.addEventListener("mousedown", (event) => this._onMouseDown(event));
        this._canvas.addEventListener("mousemove", (event) => this._onMouseMove(event));
        window.requestAnimationFrame((ts) => this._draw(ts));
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
            if (data = event.ChangeHP) {
                const FLOAT_SPEED = 10.0;

                const creature = this._cache.creatures.get(data.creature)!;
                const part = creature.parts.get(data.part)!;
                let point = this._creaturePos.get(data.creature)!;
                let float: FloatText = {
                    pos: new DOMPoint(point.x, point.y),
                    text: `${part.name}: ${data.delta} HP`,
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

    private _draw(tsMillis: DOMHighResTimeStamp) {
        this._tsMillis = tsMillis;

        this._canvas.width = this._canvas.width;
        this._ctx.translate(this._canvas.width / 2, this._canvas.height / 2);

        for (let [hex, tile] of this._cache.tiles) {
            this._drawTile(hex, tile);
        }

        this._drawHighlight(this._data.get(states.Highlight));

        const selected = this._data.get(states.Base.UI)?.selected || [];
        for (let [id, bounds] of selected) {
            for (let bound of bounds) {
                this._drawBoundary(bound);
            }
            const hex = this._cache.creatureHex.get(id);
            if (hex) {
                this._drawFocusedHex(hex);
            }
        }

        for (let [id, pos] of this._creaturePos) {
            this._drawCreature(id, pos);
        }

        for (let float of this._floatText) {
            this._ctx.save();
            this._ctx.translate(float.pos.x, float.pos.y);
            this._ctx.font = "bold 20px sans-serif";
            this._ctx.fillStyle = float.style;
            this._ctx.strokeStyle = "#000000";
            this._ctx.textAlign = "center";
            this._ctx.fillText(float.text, 0, 0);
            this._ctx.strokeText(float.text, 0, 0);
            this._ctx.restore();
        }

        for (let resolve of this._frameWaits) {
            resolve(this._tsMillis);
        }
        this._frameWaits = [];

        window.requestAnimationFrame((ts) => this._draw(ts));
    }

    private _drawTile(hex: Hex, tile: Tile) {
        this._ctx.save();

        this._pathHex(hex, HEX_SIZE);
        this._ctx.lineWidth = 1.0;
        this._ctx.strokeStyle = "#404040";
        this._ctx.fillStyle = "#FFFFFF";
        if (tile.space == "Empty") {
            this._ctx.stroke();
        } else {
            this._ctx.fill();
        }

        this._ctx.restore();
    }

    private _drawCreature(id: number, pos: DOMPointReadOnly) {
        this._ctx.save();

        this._ctx.translate(pos.x, pos.y);
        this._ctx.font = "30px sans-serif";
        this._ctx.fillStyle = "#FFFFFF";
        let text = "X";
        if (id == this._cache.playerId) {
            text = "P";
        }
        this._ctx.textAlign = "center";
        this._ctx.fillText(text, 0, this._actualTextHeight(text)/2);

        this._ctx.restore();
    }

    private _actualTextHeight(text: string): number {
        const measure = this._ctx.measureText(text);
        return measure.actualBoundingBoxAscent + measure.actualBoundingBoxDescent;
    }

    private _drawPreview(preview: readonly states.Preview[]) {
        const SCALE_MIN = 1.0;
        const SCALE_MAX = 1.2;
        const SCALE_RANGE = SCALE_MAX - SCALE_MIN;
        const SCALE_RATE = 0.25;

        const scale = SCALE_MIN + ((this._tsMillis/1000 * SCALE_RATE) % SCALE_RANGE);
        const size = HEX_SIZE * scale;

        let throb: Hex[] = [];
        for (let p of preview) {
            // TODO: show p.affects
            if (p.action.MoveCreature) {
                throb.push(p.action.MoveCreature.to);
            }
            if (p.action.HitCreature) {
                throb.push(this._cache.creatureHex.get(p.action.HitCreature.id)!);
            }
        }
        for (let hex of throb) {
            this._ctx.save();
            this._pathHex(hex, size);
            this._ctx.lineWidth = 2.0;
            this._ctx.strokeStyle = "#A0A0FF";
            this._ctx.stroke();
            this._ctx.restore();
        }
    }

    private _drawHighlight(hi?: Readonly<states.Highlight>) {
        if (!hi) { return; }
        for (let bound of hi.range) {
            this._drawBoundary(bound);
        }
        for (let hex of hi.hexes) {
            this._drawFocusedHex(hex);
        }
        this._drawPreview(hi.preview);
    }

    private _drawFocusedHex(hex: Hex) {
        this._ctx.save();

        this._pathHex(hex, HEX_SIZE);
        this._ctx.lineWidth = 2.0;
        this._ctx.strokeStyle = "#0000FF";
        this._ctx.stroke();

        this._ctx.restore();
    }

    private _drawBoundary(bound: Boundary) {
        this._ctx.save();

        let point = hexToPixel(bound.hex);
        this._ctx.translate(point.x, point.y);
        const DELTA = Math.PI/3.0;
        this._ctx.beginPath();
        for (let side of bound.sides) {
            let i;
            switch (side) {
                case "XZ": i = 0; break;
                case "YZ": i = 1; break;
                case "YX": i = 2; break;
                case "ZX": i = 3; break;
                case "ZY": i = 4; break;
                case "XY": i = 5; break;
                default: continue;
            }
            let x = Math.cos(i*DELTA)*HEX_SIZE;
            let y = Math.sin(i*DELTA)*HEX_SIZE;
            this._ctx.moveTo(x, y);
            i = i+1;
            x = Math.cos(i*DELTA)*HEX_SIZE;
            y = Math.sin(i*DELTA)*HEX_SIZE;
            this._ctx.lineTo(x, y);
        }

        this._ctx.lineWidth = 2.0;
        this._ctx.strokeStyle = "#808000";
        this._ctx.stroke();

        this._ctx.restore();
    }

    private _pathHex(hex: Hex, size: number) {
        let point = hexToPixel(hex);
        this._ctx.translate(point.x, point.y);
        const DELTA = Math.PI/3.0;
        this._ctx.beginPath();
        this._ctx.moveTo(Math.cos(0)*size, Math.sin(0)*size);
        for (let i = 1; i < 6; i++) {
            let x = Math.cos(i*DELTA)*size;
            let y = Math.sin(i*DELTA)*size;
            this._ctx.lineTo(x, y);
        }
        this._ctx.closePath();
    }

    private _onMouseDown(event: MouseEvent) {
        event.preventDefault();
        const point = this._mouseCoords(event);
        this._listener.onTileClicked(pixelToHex(point));
    }

    private _onMouseMove(event: MouseEvent) {
        const hex = pixelToHex(this._mouseCoords(event));
        if (this._cache.getTile(hex) == undefined) { return; }
        if (hex.x != this._mouseHex?.x || hex.y != this._mouseHex?.y) {
            if (this._mouseHex) {
                this._listener.onTileExited(this._mouseHex);
            }
            this._listener.onTileEntered(hex);
            this._mouseHex = hex;
        }
    }

    private _mouseCoords(event: MouseEvent): DOMPointReadOnly {
        const rect = this._canvas.getBoundingClientRect();
        const screenPoint = new DOMPointReadOnly(
            event.clientX - rect.left, event.clientY - rect.top);
        return screenPoint.matrixTransform(this._ctx.getTransform().inverse());
    }   
}

const HEX_SIZE = 30;

export interface Listener {
    onTileClicked(hex: Hex): void,
    onTileEntered(hex: Hex): void,
    onTileExited(hex: Hex): void,
}

type Constructor = new (...args: any[]) => any;

export interface DataQuery {
    get<T extends Constructor>(key: T): Readonly<InstanceType<T>> | undefined;
}

export class WorldCache {
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

export interface FloatText {
    text: string,
    pos: DOMPoint,
    style: string,
}

function hexToPixel(hex: Hex): DOMPointReadOnly {
    const x = HEX_SIZE * 3/2 * hex.x;
    const y = HEX_SIZE * (Math.sqrt(3)/2 * hex.x + Math.sqrt(3) * hex.y);
    return new DOMPointReadOnly(x, y);
}

interface FHex {
    fx: number,
    fy: number,
}

function pixelToHex(point: DOMPointReadOnly): Hex {
    const fx = (2./3 * point.x) / HEX_SIZE;
    const fy = (-1./3 * point.x + Math.sqrt(3)/3 * point.y) / HEX_SIZE;
    return hexRound({fx, fy});
}

function hexRound(hex: FHex): Hex {
    const z = -hex.fx - hex.fy;
    var rx = Math.round(hex.fx);
    var ry = Math.round(hex.fy);
    var rz = Math.round(z);

    const x_diff = Math.abs(rx - hex.fx);
    const y_diff = Math.abs(ry - hex.fy);
    const z_diff = Math.abs(rz - z);

    if (x_diff > y_diff && x_diff > z_diff) {
        rx = -ry-rz;
    } else if (y_diff > z_diff) {
        ry = -rx-rz;
    } else {
        rz = -rx-ry;
    }

    return {x: rx, y: ry};
}