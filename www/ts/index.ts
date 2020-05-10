import {enableAllPlugins} from "immer";

import {Game} from "./game";
import {LevelState} from "./states/level";
import {TitleState} from "./states/title";


enableAllPlugins();

function main() {
    const game = new Game();
    //game.stack.push(new TitleState());
    game.stack.push(new LevelState());
}

main();