import * as wasm from "../../wasm";
import {Id, Hex} from "../../wasm";
import {Focus} from "../focus";
import {Highlight} from "../highlight";
import {Stack, State} from "../stack";

export class BaseState extends State {
    onActivated() {
        this.update((draft) => {
            const focus = draft.build(Focus);
            focus.creature = {
                onEnter: (id) => this.update(draft => {
                    this._selectCreature(draft, id);
                }),
                onLeave: (id) => this.update(draft => {
                    this._unselectCreature(draft, id);
                }),
                onClick: (id) => this._clickCreature(id),
            };
            focus.part = {
                onEnter: ([cid, pid]) => this.update(draft => {
                    draft.build(Highlight).mutPartsFor(cid).inc(pid);
                }),
                onLeave: ([cid, pid]) => this.update(draft => {
                    draft.build(Highlight).mutPartsFor(cid).dec(pid);
                }),
            }
            let ui = draft.get(BaseState.UI);
            if (!ui) { return; }
            for (let id of ui.clicked.keys()) {
                this._selectCreature(draft, id);
                /*
                let range = window.game.world.getCreatureRange(id);
                let bounds = wasm.findBoundary(range);
                ui.range.set(id, bounds);
                draft.build(Highlight).creatures.inc(id);
                */
            }
        });
    }

    onDeactivated() {
        this.update(draft => {
            draft.set(Highlight);
            draft.set(Focus);
        });
    }

    onTileEntered(hex: Hex) {
        const tile = window.game.world.getTile(hex);
        if (tile?.creature != undefined) {
            const id = tile.creature;
            this.update((draft) => {
                this._selectCreature(draft, id);
            });
        }
    }

    onTileExited(hex: Hex) {
        const tile = window.game.world.getTile(hex);
        if (tile?.creature != undefined) {
            const id = tile.creature;
            this.update((draft) => {
                this._unselectCreature(draft, id);
            });
        }
    }

    onTileClicked(hex: Hex) {
        const world = window.game.world;
        let tile = world.getTile(hex);
        console.log("Tile:", hex, tile);
        if (!tile) { return; }
        if (tile.creature == undefined) {
            this.update((draft) => { this._clearClicked(draft); });
        } else {
            this._clickCreature(tile.creature);
        }
    }

    private _selectCreature(draft: Stack.Data, id: Id<wasm.Creature>) {
        const ui = draft.build(BaseState.UI);
        const highlight = draft.build(Highlight);
        if (!highlight.creatures.has(id) || !ui.range.has(id)) {
            let range = window.game.world.getCreatureRange(id);
            let bounds = wasm.findBoundary(range);
            ui.range.set(id, bounds);
        }
        highlight.creatures.inc(id);
        console.log("select", id, "count", highlight.creatures._data.get(id));
        this._buildRange(draft);
    }

    private _unselectCreature(draft: Stack.Data, id: Id<wasm.Creature>) {
        const ui = draft.build(BaseState.UI);
        const highlight = draft.build(Highlight);
        highlight.creatures.dec(id);
        console.log("unselect", id, "count", highlight.creatures._data.get(id));
        if (!highlight.creatures.has(id)) {
            console.log("delete range");
            ui.range.delete(id);
        }
        this._buildRange(draft);
    }

    private _buildRange(draft: Stack.Data) {
        const ui = draft.get(BaseState.UI);
        if (!ui) { return; }
        const totalSelected: wasm.Boundary[] = [];
        const highlight = draft.build(Highlight);
        for (let id of highlight.creatures.all() || []) {
            let sel = ui.range.get(id) || [];
            totalSelected.push(...sel);
        }
        highlight.range = totalSelected;
    }

    private _clearClicked(draft: Stack.Data) {
        const ui = draft.get(BaseState.UI);
        if (!ui) { return; }
        for (let id of ui.clicked.values() || []) {
            this._unselectCreature(draft, id);
        }
    }

    private _clickCreature(id: Id<wasm.Creature>) {
        this.update((draft) => {
            let ui = draft.build(BaseState.UI);
            if (ui.clicked.has(id)) {
                ui.clicked.delete(id);
                this._unselectCreature(draft, id);
            } else {
                ui.clicked.add(id);
                this._selectCreature(draft, id);
            }
        });
    }
}
export namespace BaseState {
    export class UI {
        [Stack.Datum] = true;
        range: Map<Id<wasm.Creature>, wasm.Boundary[]> = new Map();
        clicked: Set<Id<wasm.Creature>> = new Set();
    }
}