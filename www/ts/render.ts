import {Display, Hex, Tile} from "./data";

const HEX_SIZE = 30;

export class Engine {
    private readonly ctx: CanvasRenderingContext2D;
    constructor(private readonly canvas: HTMLCanvasElement, public display: Display) {
        this.ctx = this.canvas.getContext('2d')!;
        this.canvas.addEventListener("mousedown", (event) => this.onMouseDown(event));
        window.requestAnimationFrame(() => this.draw());
    }

    private draw() {
        this.canvas.width = this.canvas.width;
        this.ctx.translate(this.canvas.width / 2, this.canvas.height / 2);

        for (var [hex, tile] of this.display.map) {
            this.drawTile(hex, tile);
        }

        window.requestAnimationFrame(() => this.draw());
    }

    private drawTile(hex: Hex, tile: Tile) {
        this.ctx.save();

        let point = hexToPixel(hex);
        this.ctx.translate(point.x, point.y);
        const DELTA = Math.PI/3.0;
        this.ctx.beginPath();
        this.ctx.moveTo(Math.cos(0)*HEX_SIZE, Math.sin(0)*HEX_SIZE);
        for (let i = 1; i < 6; i++) {
            let x = Math.cos(i*DELTA)*HEX_SIZE;
            let y = Math.sin(i*DELTA)*HEX_SIZE;
            this.ctx.lineTo(x, y);
        }
        this.ctx.closePath();
        this.ctx.lineWidth = 1.0;
        this.ctx.strokeStyle = "#FFFFFF";
        this.ctx.stroke();

        this.ctx.restore();
    }

    private onMouseDown(event: MouseEvent) {
        console.log("click!");
        console.log(event);
        const point = this.mouseCoords(event);
        console.log(point);
        console.log(pixelToHex(point));
    }

    private mouseCoords(event: MouseEvent): DOMPointReadOnly {
        const rect = this.canvas.getBoundingClientRect();
        const screenPoint = new DOMPointReadOnly(
            event.clientX - rect.left, event.clientY - rect.top);
        return screenPoint.matrixTransform(this.ctx.getTransform().inverse());
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
    return hexRound({x: x, y: y});
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