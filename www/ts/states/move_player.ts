import * as wasm from "../../wasm";
import {Hex} from "../../wasm";

import {Highlight} from "../stack/highlight";
import {Preview} from "../stack/preview";
import {State} from "../stack";

import {UpdateState} from "./update";

export class MovePlayerState extends State {
    private _hexes: Hex[] = [];
    private _range: wasm.Boundary[] = [];
    private _from!: Hex;
    private _mp!: number;
    constructor() { super() }

    onPushed() {
        const world = window.game.world;
        const playerId = world.playerId;
        this._hexes = world.getCreatureRange(playerId);
        this._range = wasm.findBoundary(this._hexes);
        this._from = world.getCreatureHex(playerId)!;
        this._mp = world.getCreature(playerId)!.curMp;
        this.update((draft) => { draft.build(Highlight).range = this._range; });
    }
    onTileEntered(hex: Hex) {
        const world = window.game.world;
        const events = world.simulateMove(hex);
        let lastMove: Hex | undefined;
        for (let event of events) {
            let move = event.CreatureMoved;
            if (move) {
                if (move.id != world.playerId) { continue; }
                lastMove = move.to;
            }
        }
        let shade: Hex[];
        if (lastMove) {
            shade = world.shadeFrom(lastMove);
        } else {
            shade = [];
        }
        this.update((draft) => {
            draft.build(Preview).setEvents(events);
            draft.build(Highlight).shade = shade;
        });
    }
    onTileClicked(hex: Hex) {
        if (!this._hexes.some((h) => h.x == hex.x && h.y == hex.y)) { return; }
        const [next, events] = window.game.world.movePlayer(hex);
        window.game.stack.swap(new UpdateState(events, next));
    }
}