import {enableAllPlugins} from "immer";

import {Game} from "./game";
import {BaseState} from "./states/base";

enableAllPlugins();

function main() {
    const game = new Game();
    game.stack.push(new BaseState());
}

main();