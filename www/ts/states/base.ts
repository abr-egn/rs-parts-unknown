import * as wasm from "../../wasm";
import {Id, Hex} from "../../wasm";

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
                draft.build(BaseState.UI).hovered.add(id);
            });
        }
    }
    onTileExited(hex: Hex) {
        const tile = window.game.world.getTile(hex);
        if (tile?.creature != undefined) {
            const id = tile.creature;
            this.update((draft) => {
                draft.build(BaseState.UI).hovered.delete(id);
            });
        }
    }
    onTileClicked(hex: Hex) {
        const world = window.game.world;
        let tile = world.getTile(hex);
        console.log("Tile:", hex, tile);
        if (!tile) { return; }
        if (tile.creature == undefined) {
            this.update((draft) => { draft.get(BaseState.UI)?.selected.clear(); });
        } else {
            const shift = window.game.key("ShiftLeft") || window.game.key("ShiftRight");
            this.update((draft) => {
                let ui = draft.build(BaseState.UI);
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
export namespace BaseState {
    export class UI {
        [Stack.Datum] = true;
        selected: Map<Id<wasm.Creature>, wasm.Boundary[]> = new Map();
        hovered: Set<Id<wasm.Creature>> = new Set();
    }
}