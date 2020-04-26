import {immerable} from "immer";

import * as wasm from "../../wasm";
import {Hex} from "../../wasm";
import {partToTarget} from "../extra";
import {Preview} from "../preview";
import {Stack, State} from "../stack";

export class TargetPartState extends State {
    constructor(
        private _inPlay: wasm.InPlay,
        private _hex: Hex,
        private _creature: wasm.Creature,
    ) { super(); }

    onPushed() {
        const callbacks = {
            onSelect: (part: wasm.Part) => {
                window.game.stack.pop(new TargetPartState.Select(part));
            },
            onHoverEnter: (part: wasm.Part) => {
                const target = partToTarget(part);
                const events = this._inPlay.simulate(window.game.world, target);
                this.update((draft) => {
                    draft.build(Preview).setEvents(events);
                });
            },
            onHoverLeave: () => {
                this.update((draft) => {
                    draft.build(Preview).setEvents([]);
                });
            },
        };
        const targets: [wasm.Part, boolean][] = [];
        for (let part of this._creature.parts.values()) {
            const target = partToTarget(part);
            const canPlay = this._inPlay.targetValid(window.game.world, target);
            targets.push([part, canPlay]);
        }
        targets.sort((a, b) => a[0].id - b[0].id);

        this.update((draft) => {
            draft.set(TargetPartState.UI, this._hex, targets, callbacks);
        });
    }

    onTileClicked(_hex: Hex) {
        window.game.stack.pop(new TargetPartState.Cancel());
    }
}
export namespace TargetPartState {
    export class UI {
        [Stack.Datum] = true;
        [immerable] = true;

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