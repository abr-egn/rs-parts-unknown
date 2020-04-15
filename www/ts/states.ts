import {
    Action, Behavior, Boundary, Card, Creature, Event, Hex, Id, World,
    findBoundary,
} from "../wasm";
import {FloatText, hexToPixel} from "./draw";
import {State} from "./stack";

export class Base extends State {
    onTileClicked(hex: Hex) {
        const world = window.game.world;
        let tile = world.getTile(hex);
        console.log("Tile:", hex, tile);
        if (!tile) { return; }
        if (tile.creature == undefined) {
            this.update((draft) => { draft.get(Base.UI)?.selected.clear(); });
        } else {
            const shift = window.game.key("ShiftLeft") || window.game.key("ShiftRight");
            this.update((draft) => {
                let ui = draft.build(Base.UI);
                if (!shift) {
                    ui.selected.clear();
                }
                const id: Id<Creature> = tile!.creature!;
                if (ui.selected.has(id)) {
                    ui.selected.delete(id);
                } else {
                    let range = world.getCreatureRange(id);
                    let bounds = findBoundary(range);
                    ui.selected.set(id, bounds);
                }
            });
        }
    }
}
export namespace Base {
    export class UI {
        selected: Map<Id<Creature>, Boundary[]> = new Map();
    }
}

export type Stat = "AP" | "MP";

type StatMap = Map<Id<Creature>, Map<Stat, number>>;

export class Highlight {
    hexes: Hex[] = [];
    range: Boundary[] = [];

    private _stats: StatMap = new Map();
    private _float: FloatText[] = [];
    private _preview: Readonly<Preview[]> = [];

    get preview(): Readonly<Preview[]> { return this._preview; }
    set preview(value: Readonly<Preview[]>) {
        this._preview = value;
        this._stats = new Map();
        this._float = [];
        for (let prev of this._preview) {
            let act;
            if (act = prev.action.ToCreature) {
                let tc;
                if (tc = act.action.GainAP) {
                    this._addDelta(act.id, "AP", tc.ap);
                } else if (tc = act.action.SpendAP) {
                    this._addDelta(act.id, "AP", -tc.ap);
                } else if (tc = act.action.GainMP) {
                    this._addDelta(act.id, "MP", tc.mp);
                } else if (tc = act.action.SpendMP) {
                    this._addDelta(act.id, "MP", -tc.mp);
                }
            }/* TODO(hit preview)
            else if (act = prev.action.HitCreature) {
                const hex = window.game.world.getCreatureHex(act.id)!;
                this._float.push({
                    text: `-${act.damage} HP`,
                    pos: hexToPixel(hex),
                    style: "#FF0000",
                });
            }
            */
        }
    }
    get stats(): Readonly<StatMap> { return this._stats; }
    get float(): Readonly<FloatText[]> { return this._float; }

    private _addDelta(id: Id<Creature>, stat: Stat, delta: number) {
        let c = this.stats.get(id);
        if (!c) {
            c = new Map();
            this.stats.set(id, c);
        }
        let oldDelta = c.get(stat) || 0;
        c.set(stat, oldDelta + delta);
    }
}

export interface Preview {
    action: Action,
    affects: string[],
}

export interface StatPreview {
    stat: string,
    value: number,
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
            return;
        }
        const range = this._behavior.range(world);
        // Base initial highlight on player location
        const highlight = this._behavior.highlight(
            world, world.getCreatureHex(world.playerId)!);
        this.update((draft) => {
            draft.set(PlayCard.UI, this._card);
            const hi = draft.build(Highlight);
            hi.hexes = highlight;
            hi.range = findBoundary(range);
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
            // TODO: select creature in hex
            preview.push(makePreview({
                ToCreature: {
                    id: world.playerId,
                    action: { SpendAP: { ap: this._card.apCost } }
                }
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
        });
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
    private _hexes: Hex[] = [];
    private _range: Boundary[] = [];
    private _from!: Hex;
    private _mp!: number;
    constructor() { super() }

    onPushed() {
        const world = window.game.world;
        const playerId = world.playerId;
        this._hexes = world.getCreatureRange(playerId);
        this._range = findBoundary(this._hexes);
        this._from = world.getCreatureHex(playerId)!;
        this._mp = world.getCreature(playerId)!.curMp;
        this.update((draft) => { draft.build(Highlight).range = this._range; });
    }
    onTileEntered(hex: Hex) {
        const world = window.game.world;
        const path = world.path(this._from, hex);
        const preview: Preview[] = [];
        const mpCost = Math.min(Math.max(0, path.length-1), this._mp);
        if (mpCost > 0) {
            preview.push(makePreview({
                ToCreature: {
                    id: world.playerId,
                    action: { SpendMP: { mp: mpCost } },
                }
            }));
        }
        for (let hex of path.slice(0, this._mp+1)) {
            preview.push(makePreview({
                MoveCreature: { id: world.playerId, to: hex }
            }));
        }
        this.update((draft) => {
            const hi = draft.build(Highlight);
            hi.preview = preview;
        });
    }
    onTileClicked(hex: Hex) {
        if (!this._hexes.some((h) => h.x == hex.x && h.y == hex.y)) { return; }
        const [next, events] = window.game.world.movePlayer(hex);
        window.game.stack.swap(new Update(events, next));
    }
}