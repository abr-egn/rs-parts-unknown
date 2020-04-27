import {immerable} from "immer";

import {FloatText} from "../../tsx/float";
import * as wasm from "../../wasm";
import {hexToPixel} from "../draw";
import {Preview} from "../stack/preview";
import {Stack, State} from "../stack";
import {GameOverState} from "./game_over";

export class UpdateState extends State {
    constructor(
        private _events: wasm.Event[],
        private _nextWorld: wasm.World,
        private _isEndTurn?: boolean,
    ) { super(); }

    async onPushed() {
        this.update((draft) => {
            draft.set(Preview);  // clear preview state
            draft.build(UpdateState.UI).isEndTurn = Boolean(this._isEndTurn);
        });
        for (let event of this._events) {
            await this._animateEvent(event);
        }
        await window.game.updateWorld(this._nextWorld);
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

    private async _animateEvent(event: wasm.Event) {
        const board = window.game.board;
        this.update((draft) => {
            draft.build(Preview).addStats(event);
        });
        let data;
        if (data = event.CreatureMoved) {
            await board.moveCreatureTo(data.id, hexToPixel(data.to))
        }
        const float = Preview.float(event);
        if (float) {
            float.style!.animationName = "floatLift";
            window.game.addFloat(float);
            setTimeout(() => window.game.deleteFloat(float), 2000);
            await new Promise(f => setTimeout(f, 500));
        }
    }
}
export namespace UpdateState {
    export class UI {
        [Stack.Datum] = true;
        [immerable] = true;
        //float: FloatText.ItemSet = new FloatText.ItemSet();
        isEndTurn: boolean = false;
    }
}