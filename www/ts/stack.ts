import {Hex} from "./types";

export interface StateUI {
    active: boolean,
}

export class State<T extends StateUI = StateUI> {
    constructor(init: any = {}) {
        init.active = false;
        this.updateUI(_ => init);
    }

    onPushed() {}
    onPopped() {}
    onActivated() {}
    onDeactivated() {}
    onTileClicked(hex: Hex) {}
    onTileEntered(hex: Hex) {}
    onTileExited(hex: Hex) {}

    updateUI(update: (draft: T) => void) {
        const key = this.constructor as StateKey<T>;
        window.game.updateUI(key, update);
    }

    _onActivated() {
        this.updateUI(ui => { ui.active = true; });
        this.onActivated();
    }

    _onDeactivated() {
        this.updateUI(ui => { ui.active = false; });
        this.onDeactivated();
    }
}

export type StateKey<T extends StateUI> = {
    new (...args: any[]): State<T>;
}

export class Stack {
    private _stack: State[] = [];

    push(state: State) {
        console.log("PUSH: %s", state.constructor.name);
        this._top()?._onDeactivated();
        this._stack.push(state);
        state.onPushed();
        state._onActivated();
    }
    pop() {
        if (this._stack.length < 2) {
            console.error("pop() length=", this._stack.length);
            return;
        }
        const top = this._top()!;
        console.log("POP: %s --> %s", top.constructor.name,
            this._stack[this._stack.length - 2].constructor.name);
        top._onDeactivated();
        top.onPopped();
        this._stack.pop();
        this._top()!._onActivated();
    }
    swap(state: State) {
        if (this._stack.length < 1) {
            console.error("swap() length=", this._stack.length);
        }
        const top = this._top()!;
        console.log("SWAP: %s --> %s", top.constructor.name,
            state.constructor.name);
        top._onDeactivated();
        top.onPopped();
        this._stack.pop();
        this._stack.push(state);
        state.onPushed();
        state._onActivated();
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