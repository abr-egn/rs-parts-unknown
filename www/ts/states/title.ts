import {immerable} from "immer";

import {Stack, State} from "../stack";
import {LevelState} from "./level";

export class TitleState extends State {
    onActivated() {
        this.update(draft => draft.build(Title.UI, () => window.game.stack.swap(new LevelState())));
    }
}
export namespace Title {
    export class UI {
        [Stack.Datum] = true;
        [immerable] = true;
        constructor(public done: () => void) {}
    }
}