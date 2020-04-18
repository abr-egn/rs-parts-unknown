import * as React from "react";

import * as wasm from "../wasm";

import {Stat} from "../ts/highlight";

export function CreatureStats(props: {
    creature: wasm.Creature,
    stats?: Map<Stat, number>,
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
    return (<div className="uibox">
      <div style={apStyle}>AP: {props.creature.curAp + apDelta}</div>
      <div style={mpStyle}>MP: {props.creature.curMp + mpDelta}</div>
      <ul>{parts}</ul>
    </div>);
  }