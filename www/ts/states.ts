import {Behavior, World} from "../wasm";
import {State, StateUI} from "./stack";
import {Hex, Event, Card} from "./types";

export class Base extends State {
}

export interface PlayCardUI extends StateUI { card: Card }
export class PlayCard extends State<PlayCardUI> {
    private _behavior?: Behavior;
    constructor(private _card: Card) {
        super({card: _card});
    }

    onPushed() {
        const world = window.game.world;
        this._behavior = world.startPlay(this._card);
        if (!this._behavior) {
            console.log("Card did not start play:");
            console.log(this._card);
            window.game.stack.pop();
        }
        // Base initial highlight on player location
        window.game.render.highlight = this._behavior!.highlight(
            world, world.getCreatureHex(world.playerId)!);
    }

    onPopped() {
        window.game.render.highlight = [];
        window.game.render.preview = [];
        this._behavior?.free();
        this._behavior = undefined;
    }

    onTileEntered(hex: Hex) {
        let highlight: Hex[] = this._behavior!.highlight(window.game.world, hex);
        window.game.render.highlight = highlight;
        const check = window.game.world.clone();
        check.logging = false;
        if (this._behavior!.targetValid(check, hex)) {
            window.game.render.preview = this._behavior!.apply(check, hex);
        } else {
            window.game.render.preview = [];
        }
        check.free()
    }

    onTileClicked(hex: Hex) {
        if (!this._behavior!.targetValid(window.game.world, hex)) {
            return;
        }
        const world = window.game.world.clone();
        const events = this._behavior!.apply(world, hex);
        window.game.stack.swap(new Update(events, world));
    }
}

class Update extends State {
    constructor(
        private _events: Event[],
        private _nextWorld: World,
    ) { super(); }

    onPushed() {
        window.game.render.animateEvents(this._events).then(() => {
            window.game.updateWorld(this._nextWorld);
            window.game.stack.pop();
        });
    }
}

export class EndTurn extends State {
    onPushed() {
        const nextWorld = window.game.world.clone();
        let events = nextWorld.npcTurn() as Event[];
        window.game.stack.swap(new Update(events, nextWorld));
    }
}