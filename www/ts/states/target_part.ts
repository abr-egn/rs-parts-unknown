import {immerable} from "immer";

import * as wasm from "../../wasm";
import {Hex} from "../../wasm";
import {partToTarget} from "../extra";
import {Stack, State} from "../stack";
import {Focus} from "../stack/focus"
import {LevelState} from "../states/level";

export class TargetPartState extends State {
    constructor(
        private _inPlay: wasm.InPlay,
        private _hex: Hex,
        private _creature: wasm.Creature,
    ) { super(); }

    onPushed() {
        const targets: [wasm.Part, boolean][] = [];
        for (let part of this._creature.parts.values()) {
            const target = partToTarget(part);
            const world = this.stack.data.get(LevelState.Data)!.world;
            const canPlay = this._inPlay.targetValid(world, target);
            targets.push([part, canPlay]);
        }
        targets.sort((a, b) => a[0].id - b[0].id);

        this.update((draft) => {
            draft.set(TargetPartState.UI, this._hex, targets);
            const oldFocus = draft.get(Focus)?.part;
            const focus: Focus.PartHandler = Object.create(oldFocus || null);
            focus.onClick = ([cid, pid]) => {
                window.game.stack.pop(new TargetPartState.Select(cid, pid));
            };
            draft.build(Focus).part = focus;
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
        ) {}
    }
    export class Select {
        constructor(
            public cid: wasm.Id<wasm.Creature>,
            public pid: wasm.Id<wasm.Part>,
        ) {}
    }
    export class Cancel {}
}