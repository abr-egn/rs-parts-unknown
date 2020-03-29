import produce from "immer";
import * as React from "react";

import {StateKey, StateUI} from "../ts/stack";

export function index(): [JSX.Element, React.RefObject<Index>] {
    let ref = React.createRef<Index>();
    return [<Index ref={ref}/>, ref];
}

interface IndexState {
  stack: Map<StateKey<any>, any>,
}

// <params, state>
export class Index extends React.Component<{}, IndexState> {
  constructor(props: {}) {
    super(props);
    this.state = {
      stack: new Map(),
    };
  }

  updateStack<T extends StateUI>(key: StateKey<T>, update: (draft: T) => void) {
    this.setState(state => {
      state.stack.set(key, produce(state.stack.get(key), update));
    });
  }

  render() {
    return (
      <div className="center">
        <div id="leftSide" className="side"></div>
        <canvas id="mainCanvas" width="800" height="800"></canvas>
        <div className="side">
          <div id="baseRight" hidden>
            <button id="endTurn">End Turn</button>
          </div>
        </div>
      </div>
    );
  }
}