import {Hex} from "../wasm";
import {UiData} from "./ui_data";

export class Active {
    constructor(public state: State) {}
    is(c: new (...args: any[]) => any): boolean {
        return this.state.constructor == c;
    }
}

const LOGGING = false;

export class State {
    private _data?: DataUpdate;

    onPushed() {}
    onPopped() {}
    onActivated() {}
    onDeactivated() {}
    onTileClicked(hex: Hex) {}
    onTileEntered(hex: Hex) {}
    onTileExited(hex: Hex) {}

    update(update: (draft: UiData) => void) {
        this._data!.update(update);
    }

    _onPushed(data: DataUpdate) {
        if (LOGGING) {
            console.log("  PUSHED:", this.constructor.name);
        }
        this._data = data;
        this.onPushed();
    }

    _onPopped() {
        if (LOGGING) {
            console.log("  POPPED:", this.constructor.name);
        }
        this.onPopped();
    }

    _onActivated() {
        if (LOGGING) {
            console.log("  ACTIVATED:", this.constructor.name);
        }
        this.update((draft) => {
            draft.set(Active, this);
        });
        this.onActivated();
    }

    _onDeactivated() {
        if (LOGGING) {
            console.log("  DEACTIVATED:", this.constructor.name);
        }
        this.onDeactivated();
    }
}

export interface DataUpdate {
    update(update: (draft: UiData) => void): void;
}

export class Stack {
    private _stack: State[] = [];
    constructor(
        private _data: DataUpdate,
        private _listener: Listener,
    ) { }

    push(state: State) {
        setTimeout(() => {
            console.log("PUSH: %s", state.constructor.name);
            this._listener.prePush();
            this._top()?._onDeactivated();
            this._stack.push(state);
            state._onPushed(this._data);
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
            this._listener.postPop();
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
            this._listener.postPop();
            this._listener.prePush();
            this._stack.push(state);
            state._onPushed(this._data);
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

export interface Listener {
    prePush(): void;
    postPop(): void;
}