import {immerable} from "immer";

import * as wasm from "../../wasm";
import {Id, Hex} from "../../wasm";

import {partToTarget, creatureToTarget} from "../extra";
import {Focus} from "../stack/focus";
import {Highlight} from "../stack/highlight";
import {Preview} from "../stack/preview";
import {Stack, State} from "../stack";

import {TargetPartState} from "./target_part";
import {UpdateState} from "./update";

export class PlayCardState extends State {
    private _inPlay?: wasm.InPlay;
    constructor(
        private _creatureId: Id<wasm.Creature>,
        private _handIx: number,
    ) { super(); }

    onPushed() {
        const world = window.game.world;
        const creature = world.getCreature(this._creatureId);
        if (!creature) { throw `Invalid creature id ${this._creatureId}`; }
        if (this._handIx >= creature.hand.length) {
            throw `Invalid hand index ${this._handIx}`;
        }
        const card = creature.hand[this._handIx];
        
        this._inPlay = world.startPlay(this._creatureId, this._handIx);
        if (!this._inPlay) {
            throw `Card did not start play`;
        }
        const targetSpec = this._inPlay.getTargetSpec();
        if (targetSpec.None) {
            // TODO: preview, confirm
            const [nextWorld, events] = window.game.world.finishPlay(this._inPlay!, {None: true});
            this._inPlay = undefined;
            window.game.stack.swap(new UpdateState(events, nextWorld));
            return;
        }
        const range = this._inPlay.range(world);
        const targetCreatures: wasm.Creature[] = [];
        const targetParts: wasm.Part[] = [];
        for (let creature of world.getCreatures()) {
            const target = creatureToTarget(creature);
            if (this._inPlay.targetValid(world, target)) {
                targetCreatures.push(creature);
            }
            for (let part of creature.parts.values()) {
                const target = partToTarget(part);
                if (this._inPlay.targetValid(world, target)) {
                    targetParts.push(part);
                }
            }
        }
        this.update((draft) => {
            draft.set(PlayCardState.UI, card, (target) => this._playOnTarget(target), () => this._inPlay);
            const hi = draft.build(Highlight);
            hi.throb.clear();
            hi.range = wasm.findBoundary(range);
            for (let creature of targetCreatures) {
                hi.static.creatures.inc(creature.id);
            }
            for (let part of targetParts) {
                hi.static.mutPartsFor(part.creatureId).inc(part.id);
            }

            const focus = draft.build(Focus);
            
            focus.creature = this._creatureFocus();
            focus.part = this._partFocus();
        });
    }

    onActivated(data?: any) {
        if (!data) { return; }
        if (data instanceof TargetPartState.Select) {
            const target = partToTarget(data.part);
            this._playOnTarget(target);
        }
    }

    onPopped() {
        this._inPlay?.free();
        this._inPlay = undefined;
    }

    onTileEntered(hex: Hex) {
        const world = window.game.world;
        const creature = window.game.creatureAt(hex);
        if (!creature) { return; }  // TODO: hex targeting
        if (!this._canTargetCreature(creature)) { return; }

        const events: wasm.Event[] = [];
        const spec = this._inPlay!.getTargetSpec();
        if (spec.Creature) {
            const target = creatureToTarget(window.game.creatureAt(hex)!);
            events.push(...this._inPlay!.simulate(world, target));
        }
        this.update(draft => {
            draft.build(Highlight).throb.creatures.inc(creature.id);
            if (events) { draft.build(Preview).setEvents(events); }
        });
    }

    onTileExited(hex: Hex) {
        const creature = window.game.creatureAt(hex);
        this.update((draft) => {
            if (creature) {
                draft.build(Highlight).throb.creatures.dec(creature.id);
            }
            draft.build(Preview).setEvents([]);
        });
    }

    onTileClicked(hex: Hex) {
        if (!this._canTarget(hex)) { return; }
        const spec = this._inPlay!.getTargetSpec();
        if (spec.Part) {
            let creature = window.game.creatureAt(hex);
            if (!creature) { return; }
            window.game.stack.push(new TargetPartState(this._inPlay!, hex, creature));
        } else if (spec.Creature) {
            let creature = window.game.creatureAt(hex);
            if (!creature) { return; }
            const target = creatureToTarget(creature);
            this._playOnTarget(target);
        } else {
            throw "Unknown target spec!";
        }
    }

    private _canTarget(hex: Hex): boolean {
        const creature = window.game.creatureAt(hex);
        if (creature && this._canTargetCreature(creature)) { return true; }
        // TODO: hex targeting
        return false;
    }

    private _canTargetCreature(creature: wasm.Creature): boolean {
        const world = window.game.world;
        const spec = this._inPlay!.getTargetSpec();
        let match;
        if (match = spec.Part) {
            let found = false;
            for (let part of creature.parts.values()) {
                const target = partToTarget(part);
                if (this._inPlay!.targetValid(world, target)) {
                    found = true;
                    break;
                }
            }
            return found;
        } else if (match = spec.Creature) {
            const target = creatureToTarget(creature);
            return this._inPlay!.targetValid(world, target);
        }
        return false;
    }

    private _playOnTarget(target: wasm.Target) {
        if (!this._inPlay!.targetValid(window.game.world, target)) {
            return;
        }
        const [nextWorld, events] = window.game.world.finishPlay(this._inPlay!, target);
        this._inPlay = undefined;
        
        // The stack swap is deferred, so there's a brief window of time when
        // the current state is visible to the UI.  Set a tag so the UI
        // doesn't flicker for a frame.
        this.update((draft) => { draft.build(PlayCardState.ToUpdate); });
        window.game.stack.swap(new UpdateState(events, nextWorld));
    }

    private _creatureFocus(): Focus.Handler<Id<wasm.Creature>> {
        const valid = (id: Id<wasm.Creature>) => {
            const target = { Creature: { id } };
            if (!this._inPlay?.targetValid(window.game.world, target)) {
                return undefined;
            }
            return target;
        };
        return {
            onEnter: (id) => {
                if (valid(id)) {
                    this.update(draft => {
                        draft.build(Highlight).throb.creatures.inc(id);
                    });
                }
            },
            onLeave: (id) => {
                if (valid(id)) {
                    this.update(draft => {
                        draft.build(Highlight).throb.creatures.dec(id);
                    });
                }
            },
            onClick: (id) => {
                const target = valid(id);
                if (target) { this._playOnTarget(target); }
            },
        };
    }

    private _partFocus(): Focus.Handler<[Id<wasm.Creature>, Id<wasm.Part>]> {
        const valid = (cid: Id<wasm.Creature>, pid: Id<wasm.Part>) => {
            const target = {
                Part: {
                    creature_id: cid,
                    part_id: pid,
                }
            };
            if (!this._inPlay?.targetValid(window.game.world, target)) {
                return undefined;
            }
            return target;
        };
        return {
            onEnter: ([cid, pid]) => this.update(draft => {
                if (valid(cid, pid)) {
                    draft.build(Highlight).static.mutPartsFor(cid).inc(pid);
                }
            }),
            onLeave: ([cid, pid]) => this.update(draft => {
                if (valid(cid, pid)) {
                    draft.build(Highlight).static.mutPartsFor(cid).dec(pid);
                }
            }),
            onClick: ([cid, pid]) => {
                const target = valid(cid, pid);
                if (target) { this._playOnTarget(target); }
            },
        };
    }
}
export namespace PlayCardState {
    export class UI {
        [Stack.Datum] = true;
        [immerable] = true;

        constructor (
            public card: wasm.Card,
            public playOnTarget: (target: wasm.Target) => void,
            private _getInPlay: () => wasm.InPlay | undefined,
        ) {}
        get inPlay(): wasm.InPlay | undefined {
            return this._getInPlay();
        }
    }
    export class ToUpdate {
        [Stack.Datum] = true;
        [immerable] = true;
    }
}