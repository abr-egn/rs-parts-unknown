import produce, {Patch, applyPatches, produceWithPatches} from "immer";
import * as React from "react";

import {Card, Creature, World} from "../wasm";
import {Active, Stack} from "../ts/stack";
import * as States from "../ts/states";
import {UiState} from "../ts/ui_state";

export function index(): [JSX.Element, React.RefObject<Index>] {
    let ref = React.createRef<Index>();
    return [<Index ref={ref}/>, ref];
}

interface IndexState {
  map: UiState,
  world: World,
}

const _UNDO_COMPRESS_THRESHOLD: number = 10;

type Constructor = new (...args: any[]) => any;

export class Index extends React.Component<{}, IndexState> {
  private _pending: number = 0;
  private _onZero: (() => void)[] = [];
  private _undo: Map<any, Patch[]> = new Map();
  constructor(props: {}) {
    super(props);
    this.state = {
      map: new UiState(),
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

  get<T extends Constructor>(key: T): InstanceType<T> | undefined {
    return this.state.map.get(key);
  }

  update(token: any, update: (draft: UiState) => void) {
    ++this._pending;
    this.setState((prev: IndexState) => {
      let redo: Patch[] = [], undo: Patch[] = [];
      const next = produce(prev, (draft: IndexState) => {
        const [nm, nr, nu] = produceWithPatches(draft.map, update);
        draft.map = nm;
        redo = nr;
        undo = nu;
      });
      let oldUndo = this._undo.get(token);
      if (oldUndo == undefined) {
        oldUndo = [];
      }
      undo.push(...oldUndo);
      this._undo.set(token, undo);
      if (undo.length >= _UNDO_COMPRESS_THRESHOLD) {
        this._compressUndo(token);
      }
      return next;
    }, this._onSetState);
  }

  undo(token: any) {
    if (this._pending == 0) {
      this._undoImpl(token);
    } else {
      this._onZero.push(() => { this._undoImpl(token); });
    }
  }

  private _undoImpl(token: any) {
    const undo = this._undo.get(token);
    if (!undo) { return; }
    console.log("undo", token, undo);
    ++this._pending;
    this._undo.delete(token);
    this.setState((prev: IndexState) => {
      return produce(prev, (draft: IndexState) => {
        draft.map = applyPatches(draft.map, undo);
      });
    }, this._onSetState);
  }

  private _compressUndo(token: any) {
    const undo = this._undo.get(token);
    if (!undo) { return; }
    let [next, patches, inversePatches] = produceWithPatches(this.state.map,
      (draft: UiState) => {
        return applyPatches(draft, undo);
      });
    this._undo.set(token, patches);
  }

  setWorld(world: World) {
    this.setState(produce((draft: IndexState) => {
      draft.world = world;
    }));
  }

  render() {
    const world = this.state.world;
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