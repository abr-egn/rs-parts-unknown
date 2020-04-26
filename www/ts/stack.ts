import produce, {immerable} from "immer";

import {Hex} from "../wasm";

import {GameBoard} from "./game_board";

const LOGGING = false;

export class State {
    private _updater?: (update: (draft: Stack.Data) => void) => void;

    onPushed() {}
    onPopped() {}
    onActivated(data?: any) {}
    onDeactivated() {}
    onTileClicked(hex: Hex) {}
    onTileEntered(hex: Hex) {}
    onTileExited(hex: Hex) {}

    update(update: (draft: Stack.Data) => void) {
        this._updater!(update);
    }

    _onPushed(updater: (update: (draft: Stack.Data) => void) => void) {
        if (LOGGING) {
            console.log("  PUSHED:", this.constructor.name);
        }
        this._updater = updater;
        this.onPushed();
    }

    _onPopped() {
        if (LOGGING) {
            console.log("  POPPED:", this.constructor.name);
        }
        this.onPopped();
    }

    _onActivated(data?: any) {
        if (LOGGING) {
            console.log("  ACTIVATED:", this.constructor.name);
        }
        this.update((draft) => {
            draft.set(Stack.Active, this);
        });
        this.onActivated(data);
    }

    _onDeactivated() {
        if (LOGGING) {
            console.log("  DEACTIVATED:", this.constructor.name);
        }
        this.onDeactivated();
    }
}

export class Stack {
    private _stack: State[] = [];
    private _data: Stack.Data = new Stack.Data();
    private _oldData: Stack.Data[] = [];
    private _view: Stack.DataView;
    constructor(
        private _onUpdate: (() => void),
    ) {
        this._updater = this._updater.bind(this);
        // Constant instance that always refers to the *current* data object.
        this._view = {
            get: (key) => { return this._data.get(key); }
        };
    }

    push(state: State) {
        setTimeout(() => {
            console.log("PUSH: %s", state.constructor.name);
            this._top()?._onDeactivated();
            this._pushImpl(state);
        });
    }
    pop(data?: any) {
        setTimeout(() => {
            if (this._stack.length < 2) {
                console.error("pop() length=", this._stack.length);
                return;
            }
            const top = this._top()!;
            console.log("POP: %s --> %s", top.constructor.name,
                this._stack[this._stack.length - 2].constructor.name);
            this._popImpl();
            this._onUpdate();
            this._top()!._onActivated(data);
        });
    }
    swap(state: State, data?: any) {
        setTimeout(() => {
            if (this._stack.length < 1) {
                console.error("swap() length=", this._stack.length);
            }
            const top = this._top()!;
            console.log("SWAP: %s --> %s", top.constructor.name,
                state.constructor.name);
            this._popImpl();
            this._pushImpl(state);
            this._onUpdate();
        });
    }
    data(): Stack.DataView { return this._view; }

    boardListener(): GameBoard.Listener {
        return {
            onTileClicked: (hex: Hex) => {
                this._top()?.onTileClicked(hex);
            },
            onTileEntered: (hex: Hex) => {
                this._top()?.onTileEntered(hex);
            },
            onTileExited: (hex: Hex) => {
                this._top()?.onTileExited(hex);
            },
        };
    }

    private _top(): State | undefined {
        return this._stack[this._stack.length - 1];
    }
    private _pushImpl(state: State, data?: any) {
        this._oldData.push(this._data);
        this._stack.push(state);
        state._onPushed(this._updater);
        state._onActivated(data);
    }
    private _popImpl() {
        this._top()!._onDeactivated();
        this._top()!._onPopped();
        this._stack.pop();
        this._data = this._oldData.pop()!;
    }
    private _updater(update: (draft: Stack.Data) => void) {
        this._data = produce(this._data, update);
        this._onUpdate();
    }
}

export namespace Stack {
    export const Datum = Symbol();
    
    type Constructor = new (...args: any[]) => {
        [Stack.Datum]: boolean;
        [immerable]: boolean;
    };

    export interface DataView {
        get<C extends Constructor>(key: C): Readonly<InstanceType<C>> | undefined;
    }

    export class Data implements DataView {
        [immerable] = true;

        private _chunks: Map<any, any> = new Map();

        get<C extends Constructor>(key: C): Readonly<InstanceType<C>> | undefined {
            return this._chunks.get(key);
        }

        build<C extends Constructor>(key: C, ...args: ConstructorParameters<C>): InstanceType<C> {
            let chunk;
            if (chunk = this._chunks.get(key)) {
                return chunk;
            }
            chunk = new key(...args);
            this._chunks.set(key, chunk);
            return chunk as InstanceType<C>;
        }

        set<C extends Constructor>(key: C, ...args: ConstructorParameters<C>) {
            this._chunks.set(key, new key(...args));
        }

        delete<C extends Constructor>(key: C) {
            this._chunks.delete(key);
        }
    }

    export class Active {
        [Stack.Datum] = true;
        [immerable] = true;
    
        constructor(public state: State) {}
        is(c: new (...args: any[]) => any): boolean {
            return this.state.constructor == c;
        }
    }   
}