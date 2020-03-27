import {Display, Hex, Tile} from "./data";

const HEX_SIZE = 30;

export interface Listener {
    onTileClicked(hex: Hex): void,
    onTileEntered(hex: Hex): void,
    onTileExited(hex: Hex): void,
}

export class Render {
    public highlight: Hex[] = [];
    private readonly _ctx: CanvasRenderingContext2D;
    private _mouseHex?: Hex;
    private _tsMillis: DOMHighResTimeStamp;
    private _frameWaits: any[] = [];
    constructor(
            private readonly _canvas: HTMLCanvasElement,
            public display: Display,
            private readonly _listener: Listener) {
        this._tsMillis = performance.now();
        this._ctx = this._canvas.getContext('2d')!;
        this._canvas.addEventListener("mousedown", (event) => this._onMouseDown(event));
        this._canvas.addEventListener("mousemove", (event) => this._onMouseMove(event));
        window.requestAnimationFrame((ts) => this._draw(ts));
    }

    frame(): Promise<void> {
        return new Promise((resolve) => this._frameWaits.push(resolve));
    }

    private _draw(tsMillis: DOMHighResTimeStamp) {
        const deltaMillis = tsMillis - this._tsMillis;
        this._tsMillis = tsMillis;

        this._canvas.width = this._canvas.width;
        this._ctx.translate(this._canvas.width / 2, this._canvas.height / 2);

        for (var [hex, tile] of this.display.map) {
            this._drawTile(hex, tile);
        }
        for (var hex of this.highlight) {
            this._drawHighlight(hex, tsMillis);
        }

        for (let resolve of this._frameWaits) {
            resolve();
        }
        this._frameWaits = [];
        window.requestAnimationFrame((ts) => this._draw(ts));
    }

    private _drawTile(hex: Hex, tile: Tile) {
        this._ctx.save();

        let point = hexToPixel(hex);
        this._ctx.translate(point.x, point.y);
        const DELTA = Math.PI/3.0;
        this._ctx.beginPath();
        this._ctx.moveTo(Math.cos(0)*HEX_SIZE, Math.sin(0)*HEX_SIZE);
        for (let i = 1; i < 6; i++) {
            let x = Math.cos(i*DELTA)*HEX_SIZE;
            let y = Math.sin(i*DELTA)*HEX_SIZE;
            this._ctx.lineTo(x, y);
        }
        this._ctx.closePath();
        this._ctx.lineWidth = 1.0;
        this._ctx.strokeStyle = "#FFFFFF";
        this._ctx.fillStyle = "#FFFFFF";
        if (tile.space == "Empty") {
            this._ctx.stroke();
        } else {
            this._ctx.fill();
        }

        if (tile.creature != undefined) {
            this._ctx.font = "30px sans-serif";
            var text = "C";
            if (tile.creature == this.display.player_id) {
                text = "P";
            }
            const measure = this._ctx.measureText(text);
            const height = measure.actualBoundingBoxAscent + measure.actualBoundingBoxDescent;
            this._ctx.fillText(text, -measure.width / 2, height / 2);
        }

        this._ctx.restore();
    }

    private _drawHighlight(hex: Hex, tsMillis: DOMHighResTimeStamp) {
        this._ctx.save();

        const SCALE_MIN = 1.0;
        const SCALE_MAX = 1.2;
        const SCALE_RANGE = SCALE_MAX - SCALE_MIN;
        const SCALE_RATE = 0.25;

        const scale = SCALE_MIN + ((tsMillis/1000 * SCALE_RATE) % SCALE_RANGE);
        const size = HEX_SIZE * scale;

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
        this._ctx.lineWidth = 2.0;
        this._ctx.strokeStyle = "#FFFF00";
        this._ctx.stroke();

        this._ctx.restore();
    }

    private _onMouseDown(event: MouseEvent) {
        const point = this._mouseCoords(event);
        this._listener.onTileClicked(pixelToHex(point));
    }

    private _onMouseMove(event: MouseEvent) {
        const hex = pixelToHex(this._mouseCoords(event));
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

function pixelToHex(point: DOMPointReadOnly): Hex {
    const x = (2./3 * point.x) / HEX_SIZE;
    const y = (-1./3 * point.x + Math.sqrt(3)/3 * point.y) / HEX_SIZE;
    return hexRound({x, y});
}

function hexRound(hex: Hex): Hex {
    const z = -hex.x - hex.y;
    var rx = Math.round(hex.x);
    var ry = Math.round(hex.y);
    var rz = Math.round(z);

    const x_diff = Math.abs(rx - hex.x);
    const y_diff = Math.abs(ry - hex.y);
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