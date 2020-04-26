import {immerable} from "immer";

import * as wasm from "../wasm";
import {Stack} from "./stack";

export class Focus {
    [Stack.Datum] = true;
    [immerable] = true;

    creature: Focus.Handler<wasm.Id<wasm.Creature>> = {};
    part: Focus.Handler<wasm.Id<wasm.Part>> = {};
}
export namespace Focus {
    export interface Handler<T> {
        onEnter?: (value: T) => void;
        onLeave?: (value: T) => void;
        onClick?: (value: T) => void;
    }
}