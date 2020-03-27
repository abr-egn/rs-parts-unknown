import {container, injectable} from "tsyringe";

import {Hex} from "./data";
import {Game} from "./game";
import {State} from "./stack";

export class Base extends State {
    onTileClicked(hex: Hex) {
        console.log("base click");
        console.log(hex);
        const game = container.resolve(Game);
        const tile = game.tileAt(hex);
        if (tile == undefined) {
            return;
        }
        if (tile.creature == game.display.player_id) {
            this.stack.push(new MovePlayer(hex));
        }
    }
}

class MovePlayer extends State {
    constructor(private _from: Hex) { super(); }
}