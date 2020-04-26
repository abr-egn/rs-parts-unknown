import * as wasm from "../../wasm";
import {Id, Hex} from "../../wasm";
import {Highlight} from "../highlight";
import {Stack, State} from "../stack";

export class BaseState extends State {
    onActivated() {
        this.update((draft) => {
            let ui = draft.get(BaseState.UI);
            if (!ui) { return; }
            for (let id of ui.selected.keys()) {
                let range = window.game.world.getCreatureRange(id);
                let bounds = wasm.findBoundary(range);
                ui.selected.set(id, bounds);
            }
        });
    }
    onTileEntered(hex: Hex) {
        const tile = window.game.world.getTile(hex);
        if (tile?.creature != undefined) {
            const id = tile.creature;
            this.update((draft) => {
                draft.build(Highlight).creatures.inc(id);
            });
        }
    }
    onTileExited(hex: Hex) {
        const tile = window.game.world.getTile(hex);
        if (tile?.creature != undefined) {
            const id = tile.creature;
            this.update((draft) => {
                draft.build(Highlight).creatures.dec(id);
            });
        }
    }
    onTileClicked(hex: Hex) {
        const world = window.game.world;
        let tile = world.getTile(hex);
        console.log("Tile:", hex, tile);
        if (!tile) { return; }
        if (tile.creature == undefined) {
            this.update((draft) => { this._clearSelection(draft); });
        } else {
            const shift = window.game.key("ShiftLeft") || window.game.key("ShiftRight");
            this.update((draft) => {
                let ui = draft.build(BaseState.UI);
                if (!shift) {
                    this._clearSelection(draft);
                }
                const id: Id<wasm.Creature> = tile!.creature!;
                const highlight = draft.build(Highlight);
                if (ui.selected.has(id)) {
                    ui.selected.delete(id);
                    highlight.creatures.dec(id);
                } else {
                    let range = world.getCreatureRange(id);
                    let bounds = wasm.findBoundary(range);
                    ui.selected.set(id, bounds);
                    highlight.creatures.inc(id);
                }
            });
        }
    }
    private _clearSelection(draft: Stack.Data) {
        const ui = draft.get(BaseState.UI);
        if (!ui) { return; }
        for (let id of ui.selected.keys() || []) {
            draft.build(Highlight).creatures.dec(id);
        }
        ui.selected.clear();
    }
}
export namespace BaseState {
    export class UI {
        [Stack.Datum] = true;
        selected: Map<Id<wasm.Creature>, wasm.Boundary[]> = new Map();
    }
}