import produce from "immer";
import * as React from "react";

import {Card, Creature, World} from "../wasm";
import {StateKey, StateUI} from "../ts/stack";
import * as States from "../ts/states";
import {Id} from "../ts/types";
import {Game} from "../ts/game";

export function index(): [JSX.Element, React.RefObject<Index>] {
    let ref = React.createRef<Index>();
    return [<Index ref={ref}/>, ref];
}

interface IndexState {
  stack: Map<StateKey<any>, any>,
  world: World,
}

// <params, state>
export class Index extends React.Component<{}, IndexState> {
  constructor(props: {}) {
    super(props);
    this.state = {
      stack: new Map(),
      world: window.game.world,
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

  setWorld(world: World) {
    this.setState(produce((draft: IndexState) => {
      draft.world = world;
    }));
  }

  getStack<T extends StateUI>(key: StateKey<T>): T {
    return this.state.stack.get(key);
  }

  cancelPlay() {
    window.game.stack.pop();
  }

  render() {
    const base = this.getStack(States.Base);
    const play = this.getStack(States.PlayCard);
    const world = this.state.world;
    const cards = world.getCreature(world.playerId)?.getCards();
    return (
      <div className="center">
        <div id="leftSide" className="side">
          <CardList
            active={base?.active}
            cards={cards || []}
            creatureId={window.game.world.playerId}
          />
          {play?.active && <div>
            <div>Playing: {play.card.name}</div>
            <div><button onClick={this.cancelPlay}>Cancel</button></div>
          </div>}
        </div>
        <canvas id="mainCanvas" width="800" height="800"></canvas>
        <div className="side">
          <EndTurn active={base?.active}/>
        </div>
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
  creatureId: Id<Creature>,
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