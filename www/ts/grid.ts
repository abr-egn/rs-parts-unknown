import {Display, Tile} from "./data";

export class Grid {
    private _map: Array<Array<Tile>> = new Array();
    constructor(private _display: Display) {
        this._update();
    }
    get display(): Display {
        return this._display;
    }
    set display(d: Display) {
        this._display = d;
        this._update();
    }
    private _update() {
        this._map = new Array();
        for (let [hex, tile] of this._display.map) {
            
        }
    }
}