export class Engine {
    private readonly ctx: CanvasRenderingContext2D;
    constructor(private readonly canvas: HTMLCanvasElement) {
        this.ctx = this.canvas.getContext('2d', {})!;
        this.canvas.addEventListener("mousedown", (event) => this.onMouseDown(event));
        window.requestAnimationFrame(() => this.draw());
    }

    draw() {
        this.canvas.width = this.canvas.width;
        this.ctx.translate(this.canvas.width / 2, this.canvas.height / 2);

        const DELTA = Math.PI/3.0;
        const SIZE = 30;
        this.ctx.beginPath();
        this.ctx.moveTo(Math.cos(0)*SIZE, Math.sin(0)*SIZE);
        for (let i = 1; i < 6; i++) {
            let x = Math.cos(i*DELTA)*SIZE;
            let y = Math.sin(i*DELTA)*SIZE;
            this.ctx.lineTo(x, y);
        }
        this.ctx.closePath();
        this.ctx.lineWidth = 1.0;
        this.ctx.imageSmoothingEnabled = true;
        this.ctx.strokeStyle = "#FFFFFF";
        this.ctx.stroke();

        window.requestAnimationFrame(() => this.draw());
    }

    onMouseDown(event: MouseEvent) {
        console.log("click!");
        console.log(event);
    }
}