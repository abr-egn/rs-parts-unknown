import {immerable} from "immer";

import {FloatText} from "../../tsx/float";
import * as wasm from "../../wasm";
import {GameBoard} from "../game_board";
import {Stack, State} from "../stack";

export class LevelState extends State {
    onPushed() {
        const canvas = document.getElementById("mainCanvas") as HTMLCanvasElement;
        const world = new wasm.World();
        world.setTracer(new ConsoleTracer());
        const board = new GameBoard(canvas, world, this.stack.boardListener(), this.stack.data());
        const update = this.updateWorld.bind(this);
        this.update(draft => { draft.build(LevelState.UI, world, board, update); })
    }

    onPopped() {
        this.update(draft => {
            const ui = draft.getMut(LevelState.UI)!;
            // TODO: stop board
            ui.world.free();
        });
    }

    updateWorld(newWorld: wasm.World) {
        this.update(draft => {
            const ui = draft.getMut(LevelState.UI)!;
            ui.world.free();
            ui.world = newWorld;
            ui.board.updateWorld(ui.world);
        })
    }
}
export namespace LevelState {
    export class UI {
        [Stack.Datum] = true;
        [immerable] = true;
        constructor(
            public world: wasm.World,
            public board: GameBoard,
            updateWorld: (newWorld: wasm.World) => void,
        ) {}

        floats: FloatText.ItemSet = new FloatText.ItemSet();
    }
}

class ConsoleTracer implements wasm.Tracer {
    resolveAction(action: wasm.Action, events: [wasm.Event]) {
        console.log("ACTION:", action);
        for (let event of events) {
            console.log("==>", event);
        }
    }
    systemEvent(events: [wasm.Event]) {
        console.log("SYSTEM:", events[0]);
        for (let event of events.slice(1)) {
            console.log("==>", event);
        }
    }
}

class BufferTracer implements wasm.Tracer {
    private _buffer: (() => void)[] = [];
    constructor(private _wrapped: wasm.Tracer) {}
    resolveAction(action: wasm.Action, events: [wasm.Event]) {
        this._buffer.push(() => this._wrapped.resolveAction(action, events));
    }
    systemEvent(events: [wasm.Event]) {
        this._buffer.push(() => this._wrapped.systemEvent(events));
    }
    runBuffer() {
        for (let thunk of this._buffer) {
            thunk();
        }
    }
}