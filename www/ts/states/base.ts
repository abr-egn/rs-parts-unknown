import {immerable} from "immer";

import * as wasm from "../../wasm";
import {Id, Hex} from "../../wasm";
import {Focus} from "../stack/focus";
import {Highlight} from "../stack/highlight";
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
            };
            focus.part = {
                onEnter: ([cid, pid]) => this.update(draft => {
                    draft.build(Highlight).static.mutPartsFor(cid).inc(pid);
                }),
                onLeave: ([cid, pid]) => this.update(draft => {
                    draft.build(Highlight).static.mutPartsFor(cid).dec(pid);
                }),
            }
        });
    }

    onTileEntered(hex: Hex) {
        let creature = window.game.creatureAt(hex);
        if (creature) {
            let id = creature.id;
            this.update(draft => this._selectCreature(draft, id));
        }
    }

    onTileExited(hex: Hex) {
        let creature = window.game.creatureAt(hex);
        if (creature) {
            let id = creature.id;
            this.update(draft => this._unselectCreature(draft, id));
        }
    }

    onDeactivated() {
        this.update(draft => {
            draft.set(Highlight);
            draft.set(Focus);
        });
    }

    private _selectCreature(draft: Stack.Data, id: Id<wasm.Creature>) {
        const world = window.game.world;
        const highlight = draft.build(Highlight);
        highlight.range = wasm.findBoundary(world.getCreatureRange(id));
        highlight.shade = world.shadeFrom(world.getCreatureHex(id)!, id);
        highlight.static.creatures.inc(id);
    }

    private _unselectCreature(draft: Stack.Data, id: Id<wasm.Creature>) {
        const highlight = draft.build(Highlight);
        highlight.range = [];
        highlight.shade = [];
        highlight.static.creatures.dec(id);
    }
}