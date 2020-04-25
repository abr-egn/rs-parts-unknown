import * as wasm from "../../wasm";

import {Stack, State} from "../stack";

export class GameOverState extends State {
    constructor(private _state: wasm.GameState) { super(); }
    onPushed() {
        this.update((draft) => { draft.build(GameOverState.UI, this._state); });
    }
}
export namespace GameOverState {
    export class UI {
        [Stack.Datum] = true;
        constructor(public state: wasm.GameState) { }
    }
}