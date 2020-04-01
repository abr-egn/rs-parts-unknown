import {Card, Event, Hex, World} from "../wasm";
import {State, StateUI} from "./stack";

export interface BaseUI extends StateUI {
    cards: Card[],
}

export class Base extends State<BaseUI> {
    constructor() {
        super({
            cards: []
        })
        const world = this.game.world;
        const cards = world.getCreature(world.playerId)!.player!.cards;
        this.updateUI((draft) => {
            draft.cards = cards;
        });
    }
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
        const checkWorld = this.game.world.clone();
        checkWorld.logging = false;
        const events = checkWorld.movePlayer(hex.x, hex.y) as Event[];
        let canMove = true;
        let highlight: Hex[] = [];
        for (let event of events) {
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
        const nextWorld = this.game.world.clone();
        const events = nextWorld.movePlayer(hex.x, hex.y) as Event[];
        if (events.length == 0 || "Failed" in events[0].data) {
            return;
        }
        this.game.stack.swap(new Update(events, nextWorld));
    }
}

class Update extends State {
    constructor(
        private _events: Event[],
        private _nextWorld: World,
    ) { super(); }

    onPushed() {
        this.game.render.animateEvents(this._events).then(() => {
            this.game.updateWorld(this._nextWorld);
            this.game.stack.pop();
        });
    }
}

export class EndTurn extends State {
    onPushed() {
        const nextWorld = this.game.world.clone();
        let events = nextWorld.endTurn() as Event[];
        this.game.stack.swap(new Update(events, nextWorld));
    }
}