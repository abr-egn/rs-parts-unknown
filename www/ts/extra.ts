import * as wasm from "../wasm";

export function toTarget(part: wasm.Part): wasm.Target {
    return {
        Part: {
            creature_id: part.creatureId,
            part_id: part.id,
        }
    };
};