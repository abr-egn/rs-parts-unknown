import {enableAllPlugins} from "immer";

import {Game} from "./game";
import {TitleState} from "./states/title";

enableAllPlugins();

function main() {
    const game = new Game();
    game.stack.push(new TitleState());
}

main();