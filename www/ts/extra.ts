import * as wasm from "../wasm";

export function partToTarget(part: wasm.Part): wasm.Target {
    return {
        Part: {
            creature_id: part.creatureId,
            part_id: part.id,
        }
    };
};

export function creatureToTarget(creature: wasm.Creature): wasm.Target {
    return {
        Creature: {
            id: creature.id,
        }
    };
}