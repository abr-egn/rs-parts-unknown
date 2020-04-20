import * as wasm from "../wasm";
import {Id, Hex} from "../wasm";

import {Highlight, Preview} from "./highlight";
import {Stack, State} from "./stack";

export class Base extends State {
    onActivated() {
        this.update((draft) => {
            let ui = draft.get(Base.UI);
            if (!ui) { return; }
            for (let id of ui.selected.keys()) {
                let range = window.game.world.getCreatureRange(id);
                let bounds = wasm.findBoundary(range);
                ui.selected.set(id, bounds);
            }
        });
    }
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
                const id: Id<wasm.Creature> = tile!.creature!;
                if (ui.selected.has(id)) {
                    ui.selected.delete(id);
                } else {
                    let range = world.getCreatureRange(id);
                    let bounds = wasm.findBoundary(range);
                    ui.selected.set(id, bounds);
                }
            });
        }
    }
}
export namespace Base {
    export class UI {
        [Stack.Datum] = true;
        selected: Map<Id<wasm.Creature>, wasm.Boundary[]> = new Map();
    }
}

export class PlayCard extends State {
    private _inPlay?: wasm.InPlay;
    constructor(
        private _creatureId: Id<wasm.Creature>,
        private _handIx: number,
    ) { super(); }

    onPushed() {
        const world = window.game.world;
        const creature = world.getCreature(this._creatureId);
        if (!creature) { throw `Invalid creature id ${this._creatureId}`; }
        if (this._handIx >= creature.hand.length) {
            throw `Invalid hand index ${this._handIx}`;
        }
        const card = creature.hand[this._handIx];
        //this.update((draft) => { draft.build(PlayCard.UI, card).card = card; });
        
        this._inPlay = world.startPlay(this._creatureId, this._handIx);
        if (!this._inPlay) {
            throw `Card did not start play`;
        }
        const range = this._inPlay.range(world);
        this.update((draft) => {
            draft.set(PlayCard.UI, card);
            const hi = draft.build(Highlight);
            hi.hexes = [];
            hi.range = wasm.findBoundary(range);
        });
    }

    onPopped() {
        this._inPlay?.free();
        this._inPlay = undefined;
    }

    onTileEntered(hex: Hex) {
        const world = window.game.world;
        const preview: Preview[] = [];
        /* TODO(targets)
        if (this._inPlay!.targetValid(world, hex)) {
            // TODO: highlight target hex
            preview.push(Preview.make({
                ToCreature: {
                    id: world.playerId,
                    action: { SpendAP: { ap: this._inPlay!.apCost } }
                }
            }));
            const actions = this._inPlay!.preview(world, hex);
            for (let action of actions) {
                preview.push(Preview.make(action));
            }
        }
        */
        this.update((draft) => {
            const hi = draft.build(Highlight);
            hi.hexes = [];
            hi.setPreview(preview);
        });
    }

    onTileClicked(hex: Hex) {
        /* TODO(targets)
        if (!this._inPlay!.targetValid(window.game.world, hex)) {
            return;
        }
        const [nextWorld, events] = window.game.world.finishPlay(this._inPlay!, hex);
        this._inPlay = undefined;
        window.game.stack.swap(new Update(events, nextWorld));
        */
    }
}
export namespace PlayCard {
    export class UI {
        [Stack.Datum] = true;
        constructor (public card: wasm.Card) {}
    }
}

export class Update extends State {
    constructor(
        private _events: wasm.Event[],
        private _nextWorld: wasm.World,
    ) { super(); }

    async onPushed() {
        await window.game.animateEvents(this._events);
        window.game.updateWorld(this._nextWorld);
        let state: wasm.GameState;
        switch (state = window.game.world.state()) {
            case "Play": {
                window.game.stack.pop();
                break;
            }
            default: {
                window.game.stack.swap(new GameOver(state));
            }
        }
    }
}

export class EndTurn extends State {
    onPushed() {
        const [nextWorld, events] = window.game.world.npcTurn();
        window.game.stack.swap(new Update(events, nextWorld));
    }
}

export class MovePlayer extends State {
    private _hexes: Hex[] = [];
    private _range: wasm.Boundary[] = [];
    private _from!: Hex;
    private _mp!: number;
    constructor() { super() }

    onPushed() {
        const world = window.game.world;
        const playerId = world.playerId;
        this._hexes = world.getCreatureRange(playerId);
        this._range = wasm.findBoundary(this._hexes);
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
            preview.push(Preview.make({
                ToCreature: {
                    id: world.playerId,
                    action: { SpendMP: { mp: mpCost } },
                }
            }));
        }
        for (let hex of path.slice(0, this._mp+1)) {
            preview.push(Preview.make({
                MoveCreature: { id: world.playerId, to: hex }
            }));
        }
        this.update((draft) => {
            const hi = draft.build(Highlight);
            hi.setPreview(preview);
        });
    }
    onTileClicked(hex: Hex) {
        if (!this._hexes.some((h) => h.x == hex.x && h.y == hex.y)) { return; }
        const [next, events] = window.game.world.movePlayer(hex);
        window.game.stack.swap(new Update(events, next));
    }
}

export class GameOver extends State {
    constructor(private _state: wasm.GameState) { super(); }
    onPushed() {
        this.update((draft) => { draft.build(GameOver.UI, this._state); });
    }
}
export namespace GameOver {
    export class UI {
        [Stack.Datum] = true;
        constructor(public state: wasm.GameState) { }
    }
}