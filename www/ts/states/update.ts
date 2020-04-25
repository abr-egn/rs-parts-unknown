import {FloatText} from "../../tsx/float";
import * as wasm from "../../wasm";
import {hexToPixel} from "../draw";
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
        await this._animateEvents(this._events, preview);
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

    private async _animateEvents(events: wasm.Event[], preview: (ev: wasm.Event) => void) {
        const board = window.game.board;
        for (let event of events) {
            preview(event);
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
}
export namespace UpdateState {
    export class UI {
        [Stack.Datum] = true;
        float: FloatText.ItemSet = new FloatText.ItemSet();
    }
}