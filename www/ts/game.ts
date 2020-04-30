import * as React from "react";
import * as ReactDOM from "react-dom";

import {FloatText} from "../tsx/float";
import {Index} from "../tsx/index";
import * as wasm from "../wasm";
import {GameBoard} from "./game_board";
import * as stack from "./stack";
import {Preview} from "./stack/preview";
import { LevelState } from "./states/level";

declare global {
    interface Window {
        game: Game;
    }
}

export class Game {
    private _stack: stack.Stack;
    private _keys: Map<string, boolean> = new Map();
    constructor() {
        this._onUpdate = this._onUpdate.bind(this);

        this._stack = new stack.Stack(this._onUpdate);

        const canvas = document.getElementById("mainCanvas") as HTMLCanvasElement;
        canvas.width = window.innerWidth;
        canvas.height = window.innerHeight;
        window.onresize = () => {
            canvas.width = window.innerWidth;
            canvas.height = window.innerHeight;
        };
        canvas.focus();
        canvas.addEventListener('keydown', (e) => {
            this._keys.set(e.code, true);
        });
        canvas.addEventListener('keyup', (e) => {
            this._keys.set(e.code, false);
        });
        canvas.addEventListener("mousedown", (event) => {
            let level = this._stack.data.get(LevelState.Data);
            if (!level) { return; }
            level.board.onMouseDown(event);
        });
        canvas.addEventListener("mousemove", (event) => {
            let level = this._stack.data.get(LevelState.Data);
            if (!level) { return; }
            level.board.onMouseMove(event);
        });

        window.game = this;
    }

    // Accessors

    // TODO: remove this
    get stack(): stack.Stack {
        return this._stack;
    }

    key(name: string): boolean {
        return this._keys.get(name) || false;
    }

    // Private

    private _onUpdate() {
        let element = React.createElement(Index, {
            data: this._stack.data,
        }, null);
        ReactDOM.render(element, document.getElementById("root"));
    }
}