import * as wasm from "../../wasm";
import {Id, Hex} from "../../wasm";

import {partToTarget, creatureToTarget} from "../extra";
import {Highlight} from "../highlight";
import {Preview} from "../preview";
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
        this.update((draft) => {
            draft.set(PlayCardState.UI, card, (target) => this._playOnTarget(target), () => this._inPlay);
            const hi = draft.build(Highlight);
            hi.throb = [];
            hi.range = wasm.findBoundary(range);
        });
    }

    onPopped() {
        this._inPlay?.free();
        this._inPlay = undefined;
    }

    onTileEntered(hex: Hex) {
        // TODO: allow target selection via stat block UI.  Somehow.
        const world = window.game.world;
        
        const throb: Hex[] = [];
        if (this._canTarget(hex)) {
            throb.push(hex);
            const spec = this._inPlay!.getTargetSpec();
            if (spec.Creature) {
                const target = creatureToTarget(window.game.creatureAt(hex)!);
                const events = this._inPlay!.simulate(world, target);
                this.update((draft) => {
                    draft.build(Preview).setEvents(events);
                });
            }
        }
        this.update((draft) => {
            const hi = draft.build(Highlight);
            hi.throb = throb;
        });
    }

    onTileExited(hex: Hex) {
        this.update((draft) => {
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

    onActivated(data?: any) {
        if (!data) { return; }
        if (data instanceof TargetPartState.Select) {
            const target = partToTarget(data.part);
            this._playOnTarget(target);
        }
    }

    private _canTarget(hex: Hex): boolean {
        const world = window.game.world;
        const spec = this._inPlay!.getTargetSpec();
        let match;
        if (match = spec.Part) {
            let creature = window.game.creatureAt(hex);
            if (!creature) { return false; }
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
            let creature = window.game.creatureAt(hex);
            if (!creature) { return false; }
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
}
export namespace PlayCardState {
    export class UI {
        [Stack.Datum] = true;
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
    }
}