import * as React from "react";
import * as ReactDOM from "react-dom";

import * as wasm from "../wasm";

import {Highlight} from "../ts/highlight";
import {Active} from "../ts/stack";
import * as states from "../ts/states";
import {UiData} from "../ts/ui_data";

import {CreatureStats} from "./creature";
import {PlayerControls} from "./player";

export function renderIndex(world: wasm.World, data: UiData) {
    let content = <Index world={world} data={data}/>;
    ReactDOM.render(content, document.getElementById("root"));
}

function Index(props: {
  world: wasm.World,
  data: UiData,
}): JSX.Element {
  const world = props.world;
  const base = props.data.get(states.Base.UI);
  const stats = props.data.get(Highlight)?.stats;
  let creatures = [];
  if (base?.selected) {
    for (let id of base.selected.keys()) {
      const creature = world.getCreature(id);
      if (creature) {
        creatures.push(<CreatureStats key={id} creature={creature} stats={stats?.get(id)}/>);
      }
    }
  }
  const gameOverState = props.data.get(states.GameOver.UI)?.state;
  let gameOver = undefined;
  if (gameOverState) {
    gameOver = <GameOver state={gameOverState}/>;
  }
  return (
    <div>
      <canvas id="mainCanvas" tabIndex={1}></canvas>
      <div className="topleft">
        <PlayerControls
          player={world.getCreature(world.playerId)!}
          active={props.data.get(Active)}
          play={props.data.get(states.PlayCard.UI)}
          stats={stats?.get(world.playerId)}
        />
      </div>
      <div className="topright">
        {creatures}
      </div>
      {gameOver}
    </div>
  );
}

function GameOver(props: {state: wasm.GameState}): JSX.Element {
  let text: string;
  switch (props.state) {
      case "Lost": text = "You Lost!"; break;
      case "Won": text = "You Won!"; break;
      default: text = `ERROR: ${props.state}`;
  }
  return <div className="gameOver uibox">{text}</div>;
}