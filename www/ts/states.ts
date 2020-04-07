import {Behavior, World} from "../wasm";
import {State, StateUI} from "./stack";
import {
    Card, Creature, Event, Hex, Id,
    isFailure,
} from "./types";

export interface BaseUI extends StateUI { selected?: Id<Creature> }
export class Base extends State<BaseUI> {
    onTileClicked(hex: Hex) {
        let tile = window.game.world.getTile(hex);
        console.log("Tile:", hex, tile);
        if (!tile) { return; }
        if (!tile.creature || tile.creature == window.game.world.playerId) {
            window.game.render.selected = undefined;
            this.updateUI((draft) => { draft.selected = undefined; });
        } else {
            window.game.render.selected = hex;
            this.updateUI((draft) => { draft.selected = tile!.creature; });
        }
    }
    onDeactivated() {
        window.game.render.selected = undefined;
        this.updateUI((draft) => draft.selected = undefined);
    }
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
        const apWorld = window.game.world.clone();
        const apEvents = apWorld.spendAP(this._card.creatureId, this._card.apCost);
        let after = undefined;
        if (!isFailure(apEvents)) {
            const cardWorld = apWorld.clone();
            const cardEvents = this._behavior!.apply(cardWorld, hex);
            after = new Update(cardEvents, cardWorld);
        }
        window.game.stack.swap(new Update(apEvents, apWorld, after));
    }
}

class Update extends State {
    constructor(
        private _events: Event[],
        private _nextWorld: World,
        private _after?: State,
    ) { super(); }

    onPushed() {
        window.game.render.animateEvents(this._events).then(() => {
            window.game.updateWorld(this._nextWorld);
            if (this._after) {
                window.game.stack.swap(this._after);
            } else {
                window.game.stack.pop();
            }
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