import * as wasm from "../wasm";
import {Id, Hex} from "../wasm";

import {toTarget} from "./extra";
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
        
        this._inPlay = world.startPlay(this._creatureId, this._handIx);
        if (!this._inPlay) {
            throw `Card did not start play`;
        }
        const targetSpec = this._inPlay.getTargetSpec();
        if (targetSpec.None) {
            // TODO: preview, confirm
            const [nextWorld, events] = window.game.world.finishPlay(this._inPlay!, {None: true});
            this._inPlay = undefined;
            window.game.stack.swap(new Update(events, nextWorld));
            return;
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
        
        const hiHexes: Hex[] = [];
        if (this._canTarget(hex)) {
            hiHexes.push(hex);
            /* TODO: preview for direct targets
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
            */
        }
        this.update((draft) => {
            const hi = draft.build(Highlight);
            hi.hexes = hiHexes;
            hi.setPreview(preview);
        });
    }

    onTileClicked(hex: Hex) {
        if (!this._canTarget(hex)) { return; }
        const world = window.game.world;
        const spec = this._inPlay!.getTargetSpec();
        let match;
        if (match = spec.Part) {
            let creature = window.game.creatureAt(hex);
            if (!creature) { return; }
            if (creature.id == world.playerId) { return; }
            window.game.stack.push(new TargetPart(this._inPlay!, hex, creature));
        }
        // TODO: spec.Creature
    }

    onActivated(data?: any) {
        if (!data) { return; }
        if (data instanceof TargetPart.Select) {
           const target = toTarget(data.part);
            if (!this._inPlay!.targetValid(window.game.world, target)) {
                return;
            }
            const [nextWorld, events] = window.game.world.finishPlay(this._inPlay!, target);
            this._inPlay = undefined;
            window.game.stack.swap(new Update(events, nextWorld));
        }
    }

    private _canTarget(hex: Hex): boolean {
        const world = window.game.world;
        const spec = this._inPlay!.getTargetSpec();
        let match;
        if (match = spec.Part) {
            let creature = window.game.creatureAt(hex);
            if (!creature) { return false; }
            let found = false;
            for (let part of creature.parts.values()) {
                const target = toTarget(part);
                if (this._inPlay!.targetValid(world, target)) {
                    found = true;
                    break;
                }
            }
            return found;
        }
        // TODO: spec.Creature
        return false;
    }
}
export namespace PlayCard {
    export class UI {
        [Stack.Datum] = true;
        constructor (public card: wasm.Card) {}
    }
}

export class TargetPart extends State {
    constructor(
        private _inPlay: wasm.InPlay,
        private _hex: Hex,
        private _creature: wasm.Creature,
    ) { super(); }

    onPushed() {
        const callbacks = {
            onSelect: (part: wasm.Part) => {
                window.game.stack.pop(new TargetPart.Select(part));
            },
            onHoverEnter: (part: wasm.Part) => {
                const target = toTarget(part);
                const actions = this._inPlay.preview(window.game.world, target);
                const previews = actions.map(Preview.make);
                this.update((draft) => {
                    draft.build(Highlight).setPreview(previews);
                });
            },
            onHoverLeave: () => {
                this.update((draft) => {
                    draft.build(Highlight).setPreview([]);
                });
            },
        };
        const targets: [wasm.Part, boolean][] = [];
        for (let part of this._creature.parts.values()) {
            const target = toTarget(part);
            const canPlay = this._inPlay.targetValid(window.game.world, target);
            targets.push([part, canPlay]);
        }
        targets.sort((a, b) => a[0].id - b[0].id);

        // TODO: preview
        this.update((draft) => {
            draft.set(TargetPart.UI, this._hex, targets, callbacks);
        });
    }

    onTileClicked(_hex: Hex) {
        window.game.stack.pop(new TargetPart.Cancel());
    }
}
export namespace TargetPart {
    export class UI {
        [Stack.Datum] = true;
        constructor(
            public hex: Hex,
            public targets: [wasm.Part, boolean][],
            public callbacks: Callbacks,
        ) {}
    }
    export class Select {
        constructor(public part: wasm.Part) {}
    }
    export class Cancel {}
    export interface Callbacks {
        onSelect: (part: wasm.Part) => void,
        onHoverEnter: (part: wasm.Part) => void,
        onHoverLeave: () => void,
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