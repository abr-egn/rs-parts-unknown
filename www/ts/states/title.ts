import {immerable} from "immer";

import {Stack, State} from "../stack";
import {LevelState} from "./level";

export class TitleState extends State {
    onActivated() {
        this.update(draft => {
            draft.build(TitleState.UI, () => window.game.stack.swap(new LevelState()));
        });
    }
}
export namespace TitleState {
    export class UI {
        [Stack.Datum] = true;
        [immerable] = true;
        constructor(public done: () => void) {}
    }
}