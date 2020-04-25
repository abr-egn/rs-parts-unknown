import * as wasm from "../wasm";
import {Hex, Id} from "../wasm";
import {Stack} from "./stack";

import {FloatText} from "../tsx/float";

export class Highlight {
    [Stack.Datum] = true;

    hexes: Hex[] = [];
    range: wasm.Boundary[] = [];

    private _float: FloatText.ItemSet = new FloatText.ItemSet();
    private _throb: Hex[] = [];

    get float(): Readonly<FloatText.ItemId[]> { return this._float.all }
    get throb(): Readonly<Hex[]> { return this._throb; }

    setEvents(events: Readonly<wasm.Event[]>) {
        this._float = new FloatText.ItemSet();
        this._throb = [];
        for (let event of events) {
            this.addEvent(event);
        }
    }

    addEvent(event: Readonly<wasm.Event>) {
        let ev;
        if (ev = event.OnCreature) {
            let oc;
            if (oc = ev.event.OnPart) {
                let op;
                if (op = oc.event.ChangeHP) {
                    this._float.add(window.game.board.hpFloat(ev.id, oc.id, op.delta, true));
                }
            }
        } else if (ev = event.CreatureMoved) {
            this._throb.push(ev.from, ev.to);
        }
    }
}