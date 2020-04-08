import {Behavior, World} from "../wasm";
import {State, StateUI} from "./stack";
import {
    Card, Creature, Event, Hex, Id,
    isFailure,
} from "./types";

export namespace Base {
    export interface Data {
        selected: Set<Id<Creature>>,
        // Shared values for other states
        highlight: Hex[],
        preview: Event[],
    }
    export type UI = Data & StateUI;
}
export class Base extends State<Base.Data> {
    constructor() {
        super({
            selected: new Set(),
            highlight: [],
            preview: [],
        })
    }
    onTileClicked(hex: Hex) {
        let tile = window.game.world.getTile(hex);
        console.log("Tile:", hex, tile);
        if (!tile) { return; }
        if (!tile.creature || tile.creature == window.game.world.playerId) {
            this.updateUI((draft) => { draft.selected.clear(); });
        } else {
            const keys = window.game.keys;
            const shift = keys.get("ShiftLeft") || keys.get("ShiftRight");
            this.updateUI((draft) => {
                if (!shift) {
                    draft.selected.clear();
                }
                draft.selected.add(tile!.creature!);
            });
        }
    }
    onDeactivated() {
        this.updateUI((draft) => draft.selected.clear());
    }
}

export namespace PlayCard {
    export interface Data {
        card: Card,
    }
    export type UI = Data & StateUI;
}
export class PlayCard extends State<PlayCard.Data> {
    private _behavior?: Behavior;
    constructor(private _card: Card) {
        super({ card: _card });
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
        const highlight = this._behavior!.highlight(
            world, world.getCreatureHex(world.playerId)!);
        window.game.updateUI(Base, (draft) => { draft.highlight = highlight; });
    }

    onPopped() {
        window.game.updateUI(Base, (draft) => {
            draft.highlight = [];
            draft.preview = [];
        });
        this._behavior?.free();
        this._behavior = undefined;
    }

    onTileEntered(hex: Hex) {
        let highlight: Hex[] = this._behavior!.highlight(window.game.world, hex);
        const check = window.game.world.clone();
        check.logging = false;
        let preview: Event[] = [];
        if (this._behavior!.targetValid(check, hex)) {
            preview = this._behavior!.apply(check, hex);
        }
        window.game.updateUI(Base, (draft) => {
            draft.highlight = highlight;
            draft.preview = preview;
        })
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
    ) { super({}); }

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
    constructor() { super({}); }
    onPushed() {
        const nextWorld = window.game.world.clone();
        let events = nextWorld.npcTurn() as Event[];
        window.game.stack.swap(new Update(events, nextWorld));
    }
}

export class MovePlayer extends State {
    constructor() { super({}) }

    onPushed() {
        const range = window.game.world.getCreatureRange(window.game.world.playerId);
        window.game.updateUI(Base, (draft) => { draft.highlight = range; });
    }
    onPopped() {
        // TODO: use immer patches to automatically unwind changes
        window.game.updateUI(Base, (draft) => { draft.highlight = []; });
    }
    onTileEntered(hex: Hex) {
        const check = window.game.world.clone();
        check.logging = false;
        let preview = check.movePlayer(hex);
        window.game.updateUI(Base, (draft) => { draft.preview = preview; });
        check.free();
    }
}