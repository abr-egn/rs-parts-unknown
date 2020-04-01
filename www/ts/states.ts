import {Behavior, Card, Event, Hex, World} from "../wasm";
import {State, StateUI} from "./stack";

export interface BaseUI extends StateUI { cards: Card[] }
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
}

export interface PlayCardUI extends StateUI { card: Card }
export class PlayCard extends State {
    private _behavior: Behavior;
    constructor(private _card: Card) {
        super({card: _card});
        const world = this.game.world;
        this._behavior = this._card.startPlay(world, world.playerId);
        // Base initial highlight on player location
        this.game.render.highlight = this._behavior.highlight(
            world, world.getCreatureHex(world.playerId)!);
    }

    onPopped() {
        this.game.render.highlight = [];
        this.game.render.preview = [];
    }

    onTileEntered(hex: Hex) {
        let highlight: Hex[] = this._behavior.highlight(this.game.world, hex);
        this.game.render.highlight = highlight;
        const check = this.game.world.clone();
        check.logging = false;
        if (this._behavior.targetValid(check, hex)) {
            this.game.render.preview = this._behavior.apply(check, hex);
        } else {
            this.game.render.preview = [];
        }
    }

    onTileClicked(hex: Hex) {
        if (!this._behavior.targetValid(this.game.world, hex)) {
            return;
        }
        const world = this.game.world.clone();
        const events = this._behavior.apply(world, hex);
        this.game.stack.swap(new Update(events, world));
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