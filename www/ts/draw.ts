import {Boundary, Hex, Tile} from "../wasm";

export class Draw {
    constructor(private readonly _ctx: CanvasRenderingContext2D) {}

    reset() {
        const canvas = this._ctx.canvas;
        canvas.width = canvas.width;  // standard clearscreen hack
        this._ctx.translate(canvas.width / 2, canvas.height / 2);
    }

    tile(hex: Hex, tile: Tile) {
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

    creature(text: string, pos: DOMPointReadOnly) {
        this._ctx.save();

        this._ctx.translate(pos.x, pos.y);
        this._ctx.font = "30px sans-serif";
        this._ctx.fillStyle = "#FFFFFF";
        this._ctx.textAlign = "center";
        this._ctx.fillText(text, 0, this._actualTextHeight(text)/2);

        this._ctx.restore();
    }

    throb(hex: Hex, tsMillis: DOMHighResTimeStamp) {
        const SCALE_MIN = 1.0;
        const SCALE_MAX = 1.2;
        const SCALE_RANGE = SCALE_MAX - SCALE_MIN;
        const SCALE_RATE = 0.25;

        const scale = SCALE_MIN + ((tsMillis/1000 * SCALE_RATE) % SCALE_RANGE);
        const size = HEX_SIZE * scale;

        this._ctx.save();
        this._pathHex(hex, size);
        this._ctx.lineWidth = 2.0;
        this._ctx.strokeStyle = "#A0A0FF";
        this._ctx.stroke();
        this._ctx.restore();
    }

    focusedHex(hex: Hex) {
        this._ctx.save();

        this._pathHex(hex, HEX_SIZE);
        this._ctx.lineWidth = 2.0;
        this._ctx.strokeStyle = "#0000FF";
        this._ctx.stroke();

        this._ctx.restore();
    }

    boundary(bound: Boundary) {
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

    mouseCoords(event: MouseEvent): DOMPointReadOnly {
        const rect = this._ctx.canvas.getBoundingClientRect();
        const screenPoint = new DOMPointReadOnly(
            event.clientX - rect.left, event.clientY - rect.top);
        return screenPoint.matrixTransform(this._ctx.getTransform().inverse());
    }

    elementCoords(point: DOMPointReadOnly): DOMPointReadOnly {
        return point.matrixTransform(this._ctx.getTransform());
        /*
        const rect = this._ctx.canvas.getBoundingClientRect();
        return new DOMPointReadOnly(elementPoint.x + rect.left, elementPoint.y + rect.top);
        */
    }

    focus() {
        this._ctx.canvas.focus();
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

    private _actualTextHeight(text: string): number {
        const measure = this._ctx.measureText(text);
        return measure.actualBoundingBoxAscent + measure.actualBoundingBoxDescent;
    }
}

const HEX_SIZE = 30;

export function hexToPixel(hex: Hex): DOMPointReadOnly {
    const x = HEX_SIZE * 3/2 * hex.x;
    const y = HEX_SIZE * (Math.sqrt(3)/2 * hex.x + Math.sqrt(3) * hex.y);
    return new DOMPointReadOnly(x, y);
}

interface FHex {
    fx: number,
    fy: number,
}

export function pixelToHex(point: DOMPointReadOnly): Hex {
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