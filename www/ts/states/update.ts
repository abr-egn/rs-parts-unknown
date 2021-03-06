import {immerable} from "immer";

import * as wasm from "../../wasm";
import {hexToPixel} from "../draw";
import {pathCreature} from "../extra";
import {GameBoard} from "../game_board";
import {Preview} from "../stack/preview";
import {Stack, State} from "../stack";
import {GameOverState} from "./game_over";
import {LevelState} from "./level";

export class UpdateState extends State {
    constructor(
        private _events: wasm.Event[],
        private _nextWorld: wasm.World,
        private _isEndTurn?: boolean,
    ) { super(); }

    async onPushed() {
        const level = this.stack.data.get(LevelState.Data)!;
        this.update((draft) => {
            draft.set(Preview);  // clear preview state
            draft.build(UpdateState.UI).isEndTurn = Boolean(this._isEndTurn);
        });
        for (let event of this._events) {
            await this._animateEvent(event);
        }
        level.updateWorld(this._nextWorld);
        let state: wasm.GameState;
        switch (state = level.world.state()) {
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
        const level = this.stack.data.get(LevelState.Data)!;
        const board = level.board;
        this.update((draft) => {
            draft.build(Preview).addStats(event);
        });
        let data;
        if (data = event.data.Moved) {
            const motion = (board as unknown) as GameBoard.Motion;
            await motion.moveCreatureTo(pathCreature(event.target)!, hexToPixel(data.to));
        }
        const float = level.makeFloat(event);
        if (float) {
            if (!float.style) { float.style = {}; }
            float.style.animationName = "floatLift";
            this.update(draft => {
                const level = draft.mut(LevelState.Data)!;
                level.floats.add(float);
            });
            setTimeout(() => this.update(draft => {
                const level = draft.mut(LevelState.Data);
                if (level) {
                    level.floats.delete(float);
                }
            }), 2000);
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