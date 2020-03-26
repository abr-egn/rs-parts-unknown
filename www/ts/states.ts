import {Hex} from "./data";
import {State} from "./state";

export class Base extends State {
    onTileClicked(hex: Hex) {
        console.log("base click");
        console.log(hex);
    }
}