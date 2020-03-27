import {singleton} from "tsyringe";

import {Hex} from "./data";

export class State {
    onPushed() {}
    onPopped() {}
    onActivated() {}
    onDeactivated() {}
    onTileClicked(hex: Hex) {}
    onTileEntered(hex: Hex) {}
    onTileExited(hex: Hex) {}
}

@singleton()
export class Stack {
    private _stack: Array<State> = new Array();
    push(state: State) {
        console.log("PUSH: ", state.constructor.name);
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
        console.log("POP: ", top.constructor.name, " --> ",
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
        console.log("SWAP: ", top.constructor.name, " --> ",
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