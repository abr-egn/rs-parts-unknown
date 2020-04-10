import produce, {Patch, applyPatches, produceWithPatches} from "immer";
import * as React from "react";

import {Card, Creature, World} from "../wasm";
import {Active} from "../ts/stack";
import * as States from "../ts/states";
import {UiState} from "../ts/ui_state";

export function index(world: World): [JSX.Element, React.RefObject<Index>] {
    let ref = React.createRef<Index>();
    return [<Index ref={ref} world={world}/>, ref];
}

const _UNDO_COMPRESS_THRESHOLD: number = 10;

type Constructor = new (...args: any[]) => any;

interface IndexProps {
  world: World,
}
interface IndexState {
  map: UiState,
  //world: World,
}
export class Index extends React.Component<IndexProps, IndexState> {
  constructor(props: IndexProps) {
    super(props);
    this.state = {
      map: new UiState(),
    };
  }

  get<T extends Constructor>(key: T): InstanceType<T> | undefined {
    return this.state.map.get(key);
  }

  update(token: any, update: (draft: UiState) => void) {
    this.setState((prev: IndexState) => {
      return {map: produce(prev.map, update)};
    });
  }

  /*
  setWorld(world: World) {
    this.setState({world});
  }
  */

  render() {
    const world = this.props.world;
    const base = this.get(States.Base.UI);
    let creatures = [];
    if (base?.selected) {
      for (let id of base.selected.keys()) {
        const creature = world.getCreature(id);
        if (creature) {
          creatures.push(<Creature key={id} creature={creature}/>);
        }
      }
    }
    return (
      <div className="center">
        <div id="leftSide" className="side">
          <Player
            player={world.getCreature(world.playerId)!}
            active={this.get(Active)}
            play={this.get(States.PlayCard.UI)}
          />
        </div>
        <canvas id="mainCanvas" width="800" height="800" tabIndex={1}></canvas>
        <div className="side">
          {creatures}
        </div>
      </div>
    );
  }
}

function Creature(props: {
  creature: Creature,
}): JSX.Element {
  return (<div>
    <div>AP: {props.creature.curAp}</div>
    <div>MP: {props.creature.curMp}</div>
  </div>);
}

function EndTurn(props: {active: boolean}): JSX.Element {
  const onClick = () => window.game.stack.push(new States.EndTurn());
  return <button onClick={onClick} disabled={!props.active}>End Turn</button>;
}

function Player(props: {
  player: Creature,
  active?: Active,
  play?: States.PlayCard.UI,
}): JSX.Element {
  const cards: Card[] = [];
  if (props.player) {
    for (let part of props.player.parts.values()) {
      cards.push(...part.cards.values());
    }
  }

  const cancelPlay = () => window.game.stack.pop();
  const movePlayer = () => window.game.stack.push(new States.MovePlayer());

  const canPlay = props.active?.is(States.Base) || false;
  const inPlay = props.active?.is(States.PlayCard) || false;
  const canCancel = (inPlay || props.active?.is(States.MovePlayer)) || false;

  return (<div>
    Player:
    <Creature creature={props.player}/>
    <CardList
      active={canPlay}
      cards={cards}
    />
    {inPlay && <div>Playing: {props.play?.card.name}</div>}
    <EndTurn active={canPlay}/>
    {canPlay && <button onClick={movePlayer}>Move</button>}
    {canCancel &&  <div><button onClick={cancelPlay}>Cancel</button></div>}
  </div>);
}

function CardList(props: {
  active: boolean,
  cards: Card[],
}): JSX.Element {
  function startPlay(card: Card) {
    window.game.stack.push(new States.PlayCard(card));
  }
  function canPlay(card: Card): boolean {
    const world = window.game.world;
    return world.checkSpendAP(card.creatureId, card.apCost);
  }

  const list = props.cards.map((card) =>
    <li key={card.name}>
      <button
        onClick={() => startPlay(card)}
        disabled={!props.active || !canPlay(card)}>
        Play
      </button>
      [{card.apCost}] {card.name}
    </li>
  );
  return (<div>
    Cards:
    <ul>{list}</ul>
  </div>);
}