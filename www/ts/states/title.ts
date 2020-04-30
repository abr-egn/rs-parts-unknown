import {immerable} from "immer";

import {Stack, State} from "../stack";
import {BaseState} from "./base";

export class TitleState extends State {
    onActivated() {
        this.update(draft => draft.build(Title.UI, () => window.game.stack.swap(new BaseState())))
    }
}
export namespace Title {
    export class UI {
        [Stack.Datum] = true;
        [immerable] = true;
        constructor(public done: () => void) {}
    }
}