import {immerable} from "immer";

import * as wasm from "../../wasm";
import {Id} from "../../wasm";
import {Stack} from "../stack";

export class Focus {
    [Stack.Datum] = true;
    [immerable] = true;

    private _creature: HandlerWrap<Id<wasm.Creature>> = new HandlerWrap();
    private _part: HandlerWrap<[Id<wasm.Creature>, Id<wasm.Part>]> = new HandlerWrap();

    get creature(): Readonly<Focus.CreatureHandler> {
        return this._creature;
    }
    set creature(value: Readonly<Focus.CreatureHandler>) {
        this._creature = new HandlerWrap(value);
    }

    get part(): Readonly<Focus.PartHandler> {
        return this._part;
    }
    set part(value: Readonly<Focus.PartHandler>) {
        this._part = new HandlerWrap(value);
    }

    get currentCreature(): Id<wasm.Creature> | undefined {
        return this._creature.current;
    }
    get currentPart(): [Id<wasm.Creature>, Id<wasm.Part>] | undefined {
        return this._part.current;
    }
}
export namespace Focus {
    export interface Handler<T> {
        onEnter?: (value: T) => void;
        onLeave?: (value: T) => void;
        onClick?: (value: T) => void;
    }
    export type CreatureHandler = Handler<Id<wasm.Creature>>;
    export type PartHandler = Handler<[Id<wasm.Creature>, Id<wasm.Part>]>;
}

class HandlerWrap<T> implements Focus.Handler<T> {
    public current: T | undefined;
    constructor(public wrapped: Focus.Handler<T> = {}) {}
    onEnter(value: T) {
        this.current = value;
        this.wrapped.onEnter?.(value);
    }
    onLeave(value: T) {
        this.current = undefined;
        this.wrapped.onLeave?.(value);
    }
    onClick(value: T) { this.wrapped.onClick?.(value); }
}