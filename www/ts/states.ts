import {Behavior, Card, Event, World} from "../wasm";
import {State, StateUI} from "./stack";
import {Hex} from "./types";

export interface BaseUI extends StateUI { cards: Card[] }
export class Base extends State<BaseUI> {
    constructor() {
        super({
            cards: []
        })
        const world = window.game.world;
        const cards = world.getCreature(world.playerId)!.player!.cards;
        this.updateUI((draft) => {
            draft.cards = cards;
        });
    }
}

export interface PlayCardUI extends StateUI { card: Card }
export class PlayCard extends State {
    private _behavior: Behavior;
    constructor(private _card: Card) {
        super({card: _card});
        const world = window.game.world;
        this._behavior = this._card.startPlay(world, world.playerId);
        // Base initial highlight on player location
        window.game.render.highlight = this._behavior.highlight(
            world, world.getCreatureHex(world.playerId)!);
    }

    onPopped() {
        window.game.render.highlight = [];
        window.game.render.preview = [];
    }

    onTileEntered(hex: Hex) {
        let highlight: Hex[] = this._behavior.highlight(window.game.world, hex);
        window.game.render.highlight = highlight;
        const check = window.game.world.clone();
        check.logging = false;
        if (this._behavior.targetValid(check, hex)) {
            window.game.render.preview = this._behavior.apply(check, hex);
        } else {
            window.game.render.preview = [];
        }
        check.free()
    }

    onTileClicked(hex: Hex) {
        if (!this._behavior.targetValid(window.game.world, hex)) {
            return;
        }
        const world = window.game.world.clone();
        const events = this._behavior.apply(world, hex);
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