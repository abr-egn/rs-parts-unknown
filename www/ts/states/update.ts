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
            draft.build(Preview).addEvent(event, true);
        });
        let data;
        if (data = event.CreatureMoved) {
            await board.moveCreatureTo(data.id, hexToPixel(data.to))
        } else if (data = event.OnCreature) {
            let ce;
            if (ce = data.event.OnPart) {
                let pe;
                if (pe = ce.event.ChangeHP) {
                    const float = board.hpFloat(data.id, ce.id, pe.delta);
                    float.style!.animationName = "floatLift";
                    this.update((draft) => {
                        draft.build(UpdateState.UI).float.add(float);
                    });
                    await new Promise(f => setTimeout(f, 2000));
                    this.update((draft) => {
                        draft.build(UpdateState.UI).float.delete(float);
                    });
                }
            }
        }
    }
}
export namespace UpdateState {
    export class UI {
        [Stack.Datum] = true;
        [immerable] = true;
        float: FloatText.ItemSet = new FloatText.ItemSet();
        isEndTurn: boolean = false;
    }
}