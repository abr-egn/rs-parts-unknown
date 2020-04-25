import {State} from "../stack";

import {UpdateState} from "./update";

export class EndTurnState extends State {
    onPushed() {
        const [nextWorld, events] = window.game.world.npcTurn();
        window.game.stack.swap(new UpdateState(events, nextWorld, true));
    }
}
