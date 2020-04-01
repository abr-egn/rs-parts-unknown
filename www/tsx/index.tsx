import produce from "immer";
import * as React from "react";
import {container} from "tsyringe";

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
  }

  updateStack<T extends StateUI>(key: StateKey<T>, update: (draft: T) => void) {
    this.setState(produce((draft: IndexState) => {
      draft.stack.set(key, produce(draft.stack.get(key), update));
    }));
  }

  render() {
    const base = this.state.stack.get(States.Base);
    const play = this.state.stack.get(States.PlayCard);
    return (
      <div className="center">
        <div id="leftSide" className="side">
          <CardList active={base?.active} cards={base?.cards || []}/>
          {play &&
            <div>Playing: {play.card.name}</div>
          }
        </div>
        <canvas id="mainCanvas" width="800" height="800"></canvas>
        <div className="side">
          <EndTurn active={base?.active}/>
        </div>
      </div>
    );
  }
}

interface EndTurnProps {active: boolean};
class EndTurn extends React.Component<EndTurnProps, {}> {
  constructor(props: EndTurnProps) {
    super(props);
    this.onClick = this.onClick.bind(this);  // JS `this` is still terrible
  }
  onClick() {
    container.resolve(Game).stack.push(new States.EndTurn());
  }
  render() {
    return <button onClick={this.onClick} disabled={!this.props.active}>End Turn</button>
  }
}

interface CardListProps {
  active: boolean,
  cards: Card[],
};
class CardList extends React.Component<CardListProps, {}> {
  onClick(card: Card) {
    container.resolve(Game).stack.push(new States.PlayCard(card));
  }
  render() {
    const list = this.props.cards.map((card) =>
      <li key={card.name}>
        <button
          onClick={this.onClick.bind(this, card)}
          disabled={!this.props.active}>
          Play
        </button>
        [{card.apCost}] {card.name}
      </li>
    );
    return <ul>{list}</ul>;
  }
}