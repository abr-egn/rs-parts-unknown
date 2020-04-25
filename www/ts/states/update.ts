import * as wasm from "../../wasm";

import {Preview} from "../preview";
import {Stack, State} from "../stack";

import {GameOverState} from "./game_over";

export class UpdateState extends State {
    constructor(
        private _events: wasm.Event[],
        private _nextWorld: wasm.World,
    ) { super(); }

    async onPushed() {
        const preview = (ev: wasm.Event) => {
            this.update((draft) => {
                draft.build(Preview).addEvent(ev, true);
            });
        };
        this.update((draft) => {
            draft.set(Preview);
        });
        await window.game.updateWorld(this._events, this._nextWorld, preview);
        let state: wasm.GameState;
        switch (state = window.game.world.state()) {
            case "Play": {
                window.game.stack.pop();
                break;
            }
            default: {
                window.game.stack.swap(new GameOverState(state));
            }
        }
    }
}
export namespace UpdateState {
    export class UI {
        [Stack.Datum] = true;

    }
}