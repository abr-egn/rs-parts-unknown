import {container, injectable} from "tsyringe";

import {Crate} from "./crate";
import {Hex} from "./data";
import {Stack, State} from "./state";

@injectable()
export class Base extends State {
    constructor(private _stack: Stack) { super() }
    onTileClicked(hex: Hex) {
        console.log("base click");
        console.log(hex);
        this._stack.push(container.resolve(MovePlayer));
    }
}

@injectable()
class MovePlayer extends State {
    constructor(private _stack: Stack) { super() }
}