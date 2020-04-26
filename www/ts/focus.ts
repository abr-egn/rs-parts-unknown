import {immerable} from "immer";

import * as wasm from "../wasm";
import {Id} from "../wasm";
import {Stack} from "./stack";

export class Focus {
    [Stack.Datum] = true;
    [immerable] = true;

    creature: Focus.Handler<Id<wasm.Creature>> = {};
    part: Focus.Handler<[Id<wasm.Creature>, Id<wasm.Part>]> = {};
}
export namespace Focus {
    export interface Handler<T> {
        onEnter?: (value: T) => void;
        onLeave?: (value: T) => void;
        onClick?: (value: T) => void;
    }
}