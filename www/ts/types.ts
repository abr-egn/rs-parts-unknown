import {
    Behavior, Event, Hex, World,
    _find_boundary,
} from "../wasm";

/* World */

export function isFailure(events: Event[]): boolean {
    if (events.length < 1) {
        return false;
    }
    return events[0].Failed != undefined;
}

/* Behavior */

declare module "../wasm" {
    interface Behavior {
        highlight(world: World, cursor: Hex): Hex[];
        targetValid(world: World, cursor: Hex): boolean;
        apply(world: World, target: Hex): Event[];
    }
}

Behavior.prototype.highlight = Behavior.prototype._highlight;
Behavior.prototype.targetValid = Behavior.prototype._targetValid;
Behavior.prototype.apply = Behavior.prototype._apply;

/* Boundary */

export interface Boundary {
    hex: Hex,
    sides: Direction[],
}

export type Direction = "XY" | "XZ" | "YZ" | "YX" | "ZX" | "ZY";

export const find_boundary: (shape: Hex[]) => Boundary[] = _find_boundary;

/* Tracer */

export interface Tracer {
    startAction: (action: any) => void,
    modAction: (name: string, prev: any, new_: any) => void,
    resolveAction: (action: any, event: Event) => void,
}