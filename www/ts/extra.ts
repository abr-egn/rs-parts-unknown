import * as wasm from "../wasm";
import {Id} from "../wasm";

export function partToTarget(part: wasm.Part): wasm.Path {
    return {
        Part: {
            cid: part.creatureId,
            pid: part.id,
        }
    };
};

export function creatureToTarget(creature: wasm.Creature): wasm.Path {
    return {
        Creature: {
            cid: creature.id,
        }
    };
}

export function pathCreature(path: wasm.Path): Id<wasm.Creature> | undefined {
    let kind;
    if (kind = path.Creature) { return kind.cid; }
    if (kind = path.Part) { return kind.cid; }
    if (kind = path.Card) { return kind.cid; }
    return undefined;
}

export function pathPart(path: wasm.Path): Id<wasm.Part> | undefined {
    let kind;
    if (kind = path.Part) { return kind.pid; }
    return undefined;
}