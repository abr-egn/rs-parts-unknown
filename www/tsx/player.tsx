import * as React from "react";

import * as wasm from "../wasm";

import {Stat} from "../ts/highlight";
import {Active} from "../ts/stack";
import * as states from "../ts/states";

import {CardList} from "./card";
import {CreatureStats} from "./creature";

export function PlayerControls(props: {
  player: wasm.Creature,
  active?: Active,
  play?: states.PlayCard.UI,
  stats?: Map<Stat, number>,
}): JSX.Element {
  const cards: wasm.Card[] = [];
  if (props.player) {
    for (let part of props.player.parts.values()) {
      cards.push(...part.cards.values());
    }
  }

  const cancelPlay = () => window.game.stack.pop();
  const movePlayer = () => window.game.stack.push(new states.MovePlayer());

  const canPlay = props.active?.is(states.Base) || false;
  const inPlay = props.active?.is(states.PlayCard) || false;
  const canCancel = (inPlay || props.active?.is(states.MovePlayer)) || false;

  return (<div>
    Player:
    <CreatureStats creature={props.player} stats={props.stats}/>
    <CardList
      active={canPlay}
      cards={cards}
    />
    {inPlay && <div>Playing: {props.play?.card.name}</div>}
    <EndTurnButton active={canPlay}/>
    {canPlay && <button onClick={movePlayer}>Move</button>}
    {canCancel &&  <div><button onClick={cancelPlay}>Cancel</button></div>}
  </div>);
}

function EndTurnButton(props: {active: boolean}): JSX.Element {
    const onClick = () => window.game.stack.push(new states.EndTurn());
    return <button onClick={onClick} disabled={!props.active}>End Turn</button>;
}