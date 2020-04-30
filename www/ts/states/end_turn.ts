import {State} from "../stack";

import {LevelState} from "./level";
import {UpdateState} from "./update";

export class EndTurnState extends State {
    onPushed() {
        const level = this.stack.data.get(LevelState.Data)!;
        const [nextWorld, events] = level.world.npcTurn();
        this.stack.swap(new UpdateState(events, nextWorld, true));
    }
}