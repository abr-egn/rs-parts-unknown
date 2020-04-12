import * as React from "react";
import * as ReactDOM from "react-dom";

import {Card, Creature, World} from "../wasm";
import {Active} from "../ts/stack";
import * as states from "../ts/states";
import {UiData} from "../ts/ui_data";

export function renderIndex(world: World, data: UiData) {
    let content = <Index world={world} data={data}/>;
    ReactDOM.render(content, document.getElementById("root"));
}

function Index(props: {
  world: World,
  data: UiData,
}): JSX.Element {
  const world = props.world;
  const base = props.data.get(states.Base.UI);
  const stats = props.data.get(states.Highlight)?.stats;
  let creatures = [];
  if (base?.selected) {
    for (let id of base.selected.keys()) {
      const creature = world.getCreature(id);
      if (creature) {
        creatures.push(<Creature key={id} creature={creature} stats={stats?.get(id)}/>);
      }
    }
  }
  return (
    <div className="center">
      <div id="leftSide" className="side">
        <Player
          player={world.getCreature(world.playerId)!}
          active={props.data.get(Active)}
          play={props.data.get(states.PlayCard.UI)}
          stats={stats?.get(world.playerId)}
        />
      </div>
      <canvas id="mainCanvas" width="800" height="800" tabIndex={1}></canvas>
      <div className="side">
        {creatures}
      </div>
    </div>
  );
}

function Creature(props: {
  creature: Creature,
  stats?: Map<states.Stat, number>,
}): JSX.Element {
  let apDelta = props.stats?.get("AP") || 0;
  const apStyle: React.CSSProperties = {};
  if (apDelta < 0) {
    apStyle.color = "red";
  } else if (apDelta > 0) {
    apStyle.color = "green";
  }
  let mpDelta = props.stats?.get("MP") || 0;
  const mpStyle: React.CSSProperties = {};
  if (mpDelta < 0) {
    mpStyle.color = "red";
  } else if (mpDelta > 0) {
    mpStyle.color = "green";
  }
  let sorted = Array.from(props.creature.parts);
  sorted.sort(([id_a, _p_a], [id_b, _p_b]) => id_a - id_b);
  let parts = [];
  for (let [id, part] of sorted) {
    parts.push(<li key={id}>{part.name}<br/>
      HP: {part.curHp}/{part.maxHp}
    </li>);
  }
  return (<div>
    <div style={apStyle}>AP: {props.creature.curAp + apDelta}</div>
    <div style={mpStyle}>MP: {props.creature.curMp + mpDelta}</div>
    <ul>{parts}</ul>
  </div>);
}

function EndTurn(props: {active: boolean}): JSX.Element {
  const onClick = () => window.game.stack.push(new states.EndTurn());
  return <button onClick={onClick} disabled={!props.active}>End Turn</button>;
}

function Player(props: {
  player: Creature,
  active?: Active,
  play?: states.PlayCard.UI,
  stats?: Map<states.Stat, number>,
}): JSX.Element {
  const cards: Card[] = [];
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
    <Creature creature={props.player} stats={props.stats}/>
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
    window.game.stack.push(new states.PlayCard(card));
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