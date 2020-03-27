import {container, injectable} from "tsyringe";

import {Crate} from "./crate";
import {Hex} from "./data";
import {Stack, State} from "./stack";

@injectable()
export class Base extends State {
    onTileClicked(hex: Hex) {
        console.log("base click");
        console.log(hex);
        this.stack.push(container.resolve(MovePlayer).from(hex));
    }
}

@injectable()
class MovePlayer {
    constructor(private _crate: Crate) { }
    from(from: Hex) {
        return new MovePlayerState(this._crate, from);
    }
}

class MovePlayerState extends State {
    constructor(
        private _crate: Crate,
        private _from: Hex,
    ) {
        super();
    }
}