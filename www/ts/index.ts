import {enableAllPlugins} from "immer";

import {Game} from "./game";
//import {TitleState} from "./states/title";
import {LevelState} from "./states/level";

enableAllPlugins();

function main() {
    const game = new Game();
    //game.stack.push(new TitleState());
    game.stack.push(new LevelState());
}

main();