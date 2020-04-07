import {World} from "../wasm";
import {
    Boundary, Creature, Event, Hex, Id, Tile,
    find_boundary
} from "./types";
import * as States from "./states";

const HEX_SIZE = 30;

export interface Listener {
    onTileClicked(hex: Hex): void,
    onTileEntered(hex: Hex): void,
    onTileExited(hex: Hex): void,
}

export class Render {
    private readonly _ctx: CanvasRenderingContext2D;
    private _mouseHex?: Hex;
    private _tsMillis: DOMHighResTimeStamp;
    private _frameWaits: ((value: number) => void)[] = [];
    private _creaturePos: Map<Id<Creature>, DOMPointReadOnly> = new Map();
    constructor(
            private readonly _canvas: HTMLCanvasElement,
            private _world: World,
            private readonly _listener: Listener) {
        this.world = _world;  // trigger setter update
        this._tsMillis = performance.now();
        this._ctx = this._canvas.getContext('2d')!;
        this._canvas.addEventListener("mousedown", (event) => this._onMouseDown(event));
        this._canvas.addEventListener("mousemove", (event) => this._onMouseMove(event));
        window.requestAnimationFrame((ts) => this._draw(ts));
    }

    set world(d: World) {
        this._world = d;
        this._creaturePos.clear();
        for (let [id, hex] of this._world.getCreatureMap()) {
            this._creaturePos.set(id, hexToPixel(hex));
        }
    }

    async animateEvents(events: Event[]) {
        for (let event of events) {
            let move;
            if (move = event.data.CreatureMoved) {
                await this._moveCreatureTo(move.id, hexToPixel(move.to))
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
        const deltaMillis = tsMillis - this._tsMillis;
        this._tsMillis = tsMillis;

        this._canvas.width = this._canvas.width;
        this._ctx.translate(this._canvas.width / 2, this._canvas.height / 2);

        for (let [hex, tile] of this._world.getTiles()) {
            this._drawTile(hex, tile);
        }
        this._drawPreview(tsMillis);
        const highlight = window.game.getUI(States.Base)?.highlight || [];
        for (let hex of highlight) {
            this._drawHighlight(hex);
        }
        const selected = window.game.getUI(States.Base)?.selected || [];
        for (let id of selected) {
            this._drawRange(id);
            const hex = this._world.getCreatureHex(id);
            if (hex) {
                this._drawHighlight(hex);
            }
        }
        for (let [id, pos] of this._creaturePos) {
            this._drawCreature(id, pos);
        }

        for (let resolve of this._frameWaits) {
            resolve(tsMillis);
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
        if (id == this._world.playerId) {
            text = "P";
        }
        const measure = this._ctx.measureText(text);
        const height = measure.actualBoundingBoxAscent + measure.actualBoundingBoxDescent;
        this._ctx.fillText(text, -measure.width / 2, height / 2);

        this._ctx.restore();
    }

    private _drawPreview(tsMillis: DOMHighResTimeStamp) {
        const SCALE_MIN = 1.0;
        const SCALE_MAX = 1.2;
        const SCALE_RANGE = SCALE_MAX - SCALE_MIN;
        const SCALE_RATE = 0.25;

        const scale = SCALE_MIN + ((tsMillis/1000 * SCALE_RATE) % SCALE_RANGE);
        const size = HEX_SIZE * scale;

        let moves: Hex[] = [];
        const preview = window.game.getUI(States.Base)?.preview || [];
        for (let event of preview) {
            let move;
            if (move = event.data.CreatureMoved) {
                moves.push(move.to);
            }
        }
        for (let hex of moves) {
            this._ctx.save();
            this._pathHex(hex, size);
            this._ctx.lineWidth = 2.0;
            this._ctx.strokeStyle = "#A0A0FF";
            this._ctx.stroke();
            this._ctx.restore();
        }
    }

    private _drawHighlight(hex: Hex) {
        this._ctx.save();

        this._pathHex(hex, HEX_SIZE);
        this._ctx.lineWidth = 2.0;
        this._ctx.strokeStyle = "#0000FF";
        this._ctx.stroke();

        this._ctx.restore();
    }

    private _drawRange(id: Id<Creature>) {
        const shape = this._world.getCreatureRange(id);
        const bounds = find_boundary(shape);
        for (let bound of bounds) {
            this._drawBoundary(bound);
        }
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
        const point = this._mouseCoords(event);
        this._listener.onTileClicked(pixelToHex(point));
    }

    private _onMouseMove(event: MouseEvent) {
        const hex = pixelToHex(this._mouseCoords(event));
        if (this._world.getTile(hex) == undefined) { return; }
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