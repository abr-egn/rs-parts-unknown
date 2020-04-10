import {Hex} from "../wasm";
import { UiState } from "./ui_state";

export class Active {
    constructor(public state: State) {}
    is(c: new (...args: any[]) => any): boolean {
        return this.state.constructor == c;
    }
}

export class State {
    private _canUpdate: boolean = true;

    onPushed() {}
    onPopped() {}
    onActivated() {}
    onDeactivated() {}
    onTileClicked(hex: Hex) {}
    onTileEntered(hex: Hex) {}
    onTileExited(hex: Hex) {}

    update(update: (draft: UiState.Map) => void) {
        if (!this._canUpdate) {
            throw "Disallowed update";
        }
        window.game.index.update(this, update);
    }

    _onPushed() {
        //console.log("  PUSHED:", this.constructor.name);
        this.onPushed();
    }

    _onPopped() {
        //console.log("  POPPED:", this.constructor.name);
        this._canUpdate = false;
        this.onPopped();
        window.game.index.undo(this);
        this._canUpdate = true;
    }

    _onActivated() {
        //console.log("  ACTIVATED:", this.constructor.name);
        this.update((draft) => {
            draft.set(Active, this);
        });
        this.onActivated();
    }

    _onDeactivated() {
        //console.log("  DEACTIVATED:", this.constructor.name);
        this.onDeactivated();
    }
}

export class Stack {
    private _stack: State[] = [];

    push(state: State) {
        setTimeout(() => {
            console.log("PUSH: %s", state.constructor.name);
            this._top()?._onDeactivated();
            this._stack.push(state);
            state._onPushed();
            state._onActivated();
        });
    }
    pop() {
        setTimeout(() => {
            if (this._stack.length < 2) {
                console.error("pop() length=", this._stack.length);
                return;
            }
            const top = this._top()!;
            console.log("POP: %s --> %s", top.constructor.name,
                this._stack[this._stack.length - 2].constructor.name);
            top._onDeactivated();
            top._onPopped();
            this._stack.pop();
            this._top()!._onActivated();
        });
    }
    swap(state: State) {
        setTimeout(() => {
            if (this._stack.length < 1) {
                console.error("swap() length=", this._stack.length);
            }
            const top = this._top()!;
            console.log("SWAP: %s --> %s", top.constructor.name,
                state.constructor.name);
            top._onDeactivated();
            top._onPopped();
            this._stack.pop();
            this._stack.push(state);
            state._onPushed();
            state._onActivated();
        });
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