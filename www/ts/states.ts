import {
    Action, Behavior, Boundary, Card, Creature, Event, Hex, Id, World,
    findBoundary,
} from "../wasm";
import {Game} from "./game";
import {State} from "./stack";

export class Base extends State {
    onTileClicked(hex: Hex) {
        const world = window.game.world;
        let tile = world.getTile(hex);
        console.log("Tile:", hex, tile);
        if (!tile) { return; }
        if (!tile.creature || tile.creature == world.playerId) {
            this.update((draft) => { draft.get(Base.UI)?.selected.clear(); });
        } else {
            const shift = window.game.key("ShiftLeft") || window.game.key("ShiftRight");
            this.update((draft) => {
                let ui = draft.build(Base.UI);
                if (!shift) {
                    ui.selected.clear();
                }
                const id: Id<Creature> = tile!.creature!;
                let range = world.getCreatureRange(id);
                let bounds = findBoundary(range);
                ui.selected.set(tile!.creature!, bounds);
            });
        }
    }
    onDeactivated() {
        this.update((draft) => draft.get(Base.UI)?.selected.clear());
    }
}
export namespace Base {
    export class UI {
        selected: Map<Id<Creature>, Boundary[]> = new Map();
    }
}

export class Highlight {
    hexes: Hex[] = [];
    preview: Preview[] = [];
}

export interface Preview {
    action: Action,
    affects: string[],
}

export class PlayCard extends State {
    private _behavior?: Behavior;
    constructor(private _card: Card) { super(); }

    onPushed() {
        this.update((draft) => { draft.build(PlayCard.UI, this._card).card = this._card; });
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
        this.update((draft) => {
            draft.set(PlayCard.UI, this._card);
            draft.build(Highlight).hexes = highlight;
        });
    }

    onPopped() {
        this._behavior?.free();
        this._behavior = undefined;
    }

    onTileEntered(hex: Hex) {
        const world = window.game.world;
        let highlight: Hex[] = this._behavior!.highlight(world, hex);
        const preview: Preview[] = [];
        if (this._behavior!.targetValid(world, hex)) {
            preview.push(makePreview({
                SpendAP: {id: world.playerId, ap: this._card.apCost}
            }));
            const actions = this._behavior!.preview(world, hex);
            for (let action of actions) {
                preview.push(makePreview(action));
            }
        }
        this.update((draft) => {
            const hi = draft.build(Highlight);
            hi.hexes = highlight;
            hi.preview = preview;
        })
    }

    onTileClicked(hex: Hex) {
        if (!this._behavior!.targetValid(window.game.world, hex)) {
            return;
        }
        const [nextWorld, events] = window.game.world.playCard(this._card, this._behavior!, hex);
        this._behavior = undefined;
        window.game.stack.swap(new Update(events, nextWorld));
    }
}
export namespace PlayCard {
    export class UI {
        constructor (public card: Card) {}
    }
}

class Update extends State {
    constructor(
        private _events: Event[],
        private _nextWorld: World,
    ) { super(); }

    onPushed() {
        window.game.animateEvents(this._events).then(() => {
            window.game.updateWorld(this._nextWorld);
            window.game.stack.pop();
        });
    }
}

export class EndTurn extends State {
    constructor() { super(); }
    onPushed() {
        const [nextWorld, events] = window.game.world.npcTurn();
        window.game.stack.swap(new Update(events, nextWorld));
    }
}

function makePreview(act: Action): Preview {
    return {
        action: act,
        affects: window.game.world.affectsAction(act),
    };
}

export class MovePlayer extends State {
    private _range: Hex[] = [];
    private _from!: Hex;
    private _mp!: number;
    constructor() { super() }

    onPushed() {
        const world = window.game.world;
        const playerId = world.playerId;
        this._range = world.getCreatureRange(playerId);
        this._from = world.getCreatureHex(playerId)!;
        this._mp = world.getCreature(playerId)!.curMp;
        this.update((draft) => { draft.build(Highlight).hexes = this._range; });
    }
    onTileEntered(hex: Hex) {
        const world = window.game.world;
        const path = world.path(this._from, hex);
        const preview: Preview[] = [];
        if (path.length > 0) {
            preview.push(makePreview({
                SpendMP: {
                    id: world.playerId,
                    mp: Math.max(path.length, this._mp),
                }
            }));
        }
        for (let hex of path.slice(0, this._mp+1)) {
            preview.push(makePreview({
                MoveCreature: { id: world.playerId, to: hex }
            }));
        }
        this.update((draft) => { draft.build(Highlight).preview = preview; });
    }
    onTileClicked(hex: Hex) {
        if (!this._range.some((h) => h.x == hex.x && h.y == hex.y)) { return; }
        const [next, events] = window.game.world.movePlayer(hex);
        window.game.stack.swap(new Update(events, next));
    }
}