import {container} from "tsyringe";

import {Game} from "./game";
import {Hex} from "./data";

export class State {
    private _game: Game;
    constructor() {
        this._game = container.resolve(Game);
    }

    onPushed() {}
    onPopped() {}
    onActivated() {}
    onDeactivated() {}
    onTileClicked(hex: Hex) {}
    onTileEntered(hex: Hex) {}
    onTileExited(hex: Hex) {}

    get game(): Game {
        return this._game;
    }
}

export class Stack {
    private _stack: State[] = [];
    push(state: State) {
        console.log("PUSH: %s", state.constructor.name);
        this._top()?.onDeactivated();
        this._stack.push(state);
        state.onPushed();
        state.onActivated();
    }
    pop() {
        if (this._stack.length < 2) {
            console.error("pop() length=", this._stack.length);
            return;
        }
        const top = this._top()!;
        console.log("POP: %s --> %s", top.constructor.name,
            this._stack[this._stack.length - 2].constructor.name);
        top.onDeactivated();
        top.onPopped();
        this._stack.pop();
        this._top()!.onActivated();
    }
    swap(state: State) {
        if (this._stack.length < 1) {
            console.error("swap() length=", this._stack.length);
        }
        const top = this._top()!;
        console.log("SWAP: %s --> %s", top.constructor.name,
            state.constructor.name);
        top.onDeactivated();
        top.onPopped();
        this._stack.pop();
        this._stack.push(state);
        state.onPushed();
        state.onActivated();
    }
    onTileClicked(hex: Hex) {
        this._top()?.onTileClicked(hex);
    }
    onTileEntered(hex: Hex) {
        this._top()?.onTileEntered(hex);
    }
    onTileExited(hex: Hex) {
        this._top()?.onTileExited(hex);
    }
    private _top(): State | undefined {
        return this._stack[this._stack.length - 1];
    }
}