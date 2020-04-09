import produce, {Patch, applyPatches, produceWithPatches} from "immer";
import * as React from "react";

import {Card, Creature, World} from "../wasm";
import {StateKey, StateUI} from "../ts/stack";
import * as States from "../ts/states";

export function index(): [JSX.Element, React.RefObject<Index>] {
    let ref = React.createRef<Index>();
    return [<Index ref={ref}/>, ref];
}

interface IndexState {
  stack: Map<StateKey<any>, any>,
  world: World,
}

export class Index extends React.Component<{}, IndexState> {
  private _pending: number = 0;
  private _onZero: (() => void)[] = [];
  private _undo: Map<any, Patch[]> = new Map();
  constructor(props: {}) {
    super(props);
    this.state = {
      stack: new Map(),
      world: window.game.world,
    };
    this._onSetState = this._onSetState.bind(this);
  }

  private _onSetState() {
    if (--this._pending == 0) {
      for (let thunk of this._onZero) {
        thunk();
      }
      this._onZero = [];
    }
  }

  updateStack<T>(token: any, key: StateKey<T>, update: (draft: T & StateUI) => void) {
    ++this._pending;
    this.setState((prev: IndexState) => {
      const [next, patches, inversePatches] = produceWithPatches(prev, (draft: IndexState) => {
        draft.stack.set(key, produce(draft.stack.get(key), update));
      });
      let prevUndo = this._undo.get(token);
      if (prevUndo == undefined) {
        prevUndo = [];
      }
      inversePatches.push(...prevUndo);
      this._undo.set(token, inversePatches);
      return next;
    }, this._onSetState);
  }

  undoStack(token: any) {
    if (this._pending == 0) {
      this._undoStack(token);
    } else {
      this._onZero.push(() => { this._undoStack(token); });
    }
  }

  private _undoStack(token: any) {
    const undo = this._undo.get(token);
    if (!undo) { return; }
    console.log("==> undo:", token, undo);
    ++this._pending;
    this._undo.delete(token);
    this.setState((prev: IndexState) => {
      return applyPatches(prev, undo);
    }, this._onSetState);
  }

  setWorld(world: World) {
    this.setState(produce((draft: IndexState) => {
      draft.world = world;
    }));
  }

  getStack<T>(key: StateKey<T>): (T & StateUI) | undefined {
    return this.state.stack.get(key);
  }

  render() {
    const world = this.state.world;
    const base = this.getStack(States.Base);
    let creatures = [];
    if (base?.selected) {
      for (let id of base.selected) {
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
            canPlay={base?.active || false}
            play={this.getStack(States.PlayCard)}
            move={this.getStack(States.MovePlayer)}
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
  canPlay: boolean,
  play?: States.PlayCard.UI,
  move?: StateUI,
}): JSX.Element {
  const cards: Card[] = [];
  if (props.player) {
    for (let part of props.player.parts.values()) {
      cards.push(...part.cards.values());
    }
  }

  const cancelPlay = () => window.game.stack.pop();
  const movePlayer = () => window.game.stack.push(new States.MovePlayer());

  return (<div>
    Player:
    <Creature creature={props.player}/>
    <CardList
      active={props.canPlay}
      cards={cards}
    />
    {props.play?.active && <div>Playing: {props.play.card.name}</div>}
    <EndTurn active={props.canPlay}/>
    {props.canPlay && <button onClick={movePlayer}>Move</button>}
    {(props.play?.active || props.move?.active) && 
    <div><button onClick={cancelPlay}>Cancel</button></div>}
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