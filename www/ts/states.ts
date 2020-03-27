import {Hex} from "./data";
import {Stack, State} from "./state";

export class Base extends State {
    constructor(private _stack: Stack) { super() }
    onTileClicked(hex: Hex) {
        console.log("base click");
        console.log(hex);
    }
}

class MovePlayer extends State {
    constructor(private _stack: Stack) { super() }
}