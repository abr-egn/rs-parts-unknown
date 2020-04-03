import produce from "immer";
import * as React from "react";

import {Card} from "../wasm";
import {Game} from "../ts/game";
import {StateKey, StateUI} from "../ts/stack";
import * as States from "../ts/states";

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
    this.cancelPlay = this.cancelPlay.bind(this);
  }

  updateStack<T extends StateUI>(key: StateKey<T>, update: (draft: T) => void) {
    this.setState((prev: IndexState) => {
      return produce(prev, (draft: IndexState) => {
        draft.stack.set(key, produce(draft.stack.get(key), update));
      });
    })
  }

  cancelPlay() {
    window.game!.stack.pop();
  }

  render() {
    const base = this.state.stack.get(States.Base);
    const play = this.state.stack.get(States.PlayCard);
    const left = (window.game &&
      <div id="leftSide" className="side">
        <CardList
          active={base?.active}
          cards={base?.cards || []}
          creatureId={window.game.world.playerId}
        />
        {play?.active && <div>
          <div>Playing: {play.card.name}</div>
          <div><button onClick={this.cancelPlay}>Cancel</button></div>
        </div>}
      </div>
    );
    const right = (window.game &&
      <div className="side">
        <EndTurn active={base?.active}/>
      </div>
    );
    return (
      <div className="center">
        {left}
        <canvas id="mainCanvas" width="800" height="800"></canvas>
        {right}
      </div>
    );
  }
}

interface EndTurnProps {
  active: boolean,
};
class EndTurn extends React.Component<EndTurnProps, {}> {
  constructor(props: EndTurnProps) {
    super(props);
    this.onClick = this.onClick.bind(this);  // JS `this` is still terrible
  }
  onClick() {
    window.game.stack.push(new States.EndTurn());
  }
  render() {
    return <button onClick={this.onClick} disabled={!this.props.active}>End Turn</button>
  }
}

interface CardListProps {
  active: boolean,
  cards: Card[],
  creatureId: number,
};
class CardList extends React.Component<CardListProps, {}> {
  onClick(card: Card) {
    window.game.stack.push(new States.PlayCard(card));
  }
  canPlay(card: Card): boolean {
    const world = window.game.world;
    return world.checkSpendAP(this.props.creatureId, card.apCost);
  }
  render() {
    const list = this.props.cards.map((card) =>
      <li key={card.name}>
        <button
          onClick={this.onClick.bind(this, card)}
          disabled={!this.props.active || !this.canPlay(card)}>
          Play
        </button>
        [{card.apCost}] {card.name}
      </li>
    );
    return <ul>{list}</ul>;
  }
}