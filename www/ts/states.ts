import {createCheckers} from "ts-interface-checker";
import {container} from "tsyringe";

import {Hex, Meta} from "./data";
import dataTI from "./data-ti";
import {Game} from "./game";
import {State} from "./stack";

const CHECKERS = createCheckers(dataTI);

export class Base extends State {
    onTileClicked(hex: Hex) {
        console.log("base click");
        console.log(hex);
        const game = container.resolve(Game);
        if (game.tileAt(hex)?.creature == game.display.player_id) {
            this.stack.push(new MovePlayer(hex));
        }
    }
}

class MovePlayer extends State {
    constructor(private _from: Hex) { super(); }

    onPopped() {
        container.resolve(Game).engine.highlight = [];
    }

    onTileEntered(hex: Hex) {
        const game = container.resolve(Game);
        if (game.tileAt(hex) == undefined) {
            return;
        }
        game.backend.startCheck();
        const events = game.backend.movePlayer(hex.x, hex.y) as Meta[];
        game.backend.endCheck();
        console.log(events);
        let canMove = true;
        let highlight: Hex[] = [];
        for (let event of events) {
            CHECKERS.Meta.check(event);
            if ("Failed" in event.data) {
                canMove = false;
                break;
            }
            if ("CreatureMoved" in event.data) {
                highlight.push(...event.data.CreatureMoved.path);
            }
        }
        if (!canMove) {
            highlight = [];
        }
        game.engine.highlight = highlight;
    }
}