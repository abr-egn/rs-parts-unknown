import {createCheckers} from "ts-interface-checker";

import {Hex, Meta} from "./data";
import dataTI from "./data-ti";
import {State} from "./stack";

const CHECKERS = createCheckers(dataTI);

export class Base extends State {
    onTileClicked(hex: Hex) {
        if (this.game.tileAt(hex)?.creature == this.game.world.playerId) {
            this.game.stack.push(new MovePlayer(hex));
        }
    }
}

class MovePlayer extends State {
    constructor(private _from: Hex) { super(); }

    onPopped() {
        this.game.render.highlight = [];
    }

    onTileEntered(hex: Hex) {
        if (this.game.tileAt(hex) == undefined) {
            return;
        }
        this.game.backend.startCheck();
        const events = this.game.backend.movePlayer(hex.x, hex.y) as Meta[];
        this.game.backend.endCheck();
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
        this.game.render.highlight = highlight;
    }

    onTileClicked(hex: Hex) {
        if (this.game.tileAt(hex) == undefined) {
            return;
        }
        const events = this.game.backend.movePlayer(hex.x, hex.y) as Meta[];
        if (events.length == 0 || "Failed" in events[0].data) {
            return;
        }
        this.game.stack.swap(new Update(events));
    }
}

class Update extends State {
    constructor(private _events: Meta[]) { super(); }

    onPushed() {
        this.game.render.animateEvents(this._events).then(() => {
            this.game.updateWorld();
            this.game.stack.pop();
        });
    }
}

export class EndTurn extends State {
    onPushed() {
        let events = this.game.backend.endTurn() as Meta[];
        this.game.stack.swap(new Update(events));
    }
}