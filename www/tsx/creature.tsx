import * as React from "react";

import * as wasm from "../wasm";
import {Id} from "../wasm";

import {Stat} from "../ts/highlight";

export function CreatureStats(props: {
    creature: wasm.Creature,
    stats?: Map<Stat, number>,
    partHighlight?: Id<wasm.Part>,
    setPartHighlight?: (part: Id<wasm.Part> | undefined) => void,
}): JSX.Element {
    const onPartEnter = (part: Id<wasm.Part>, event: React.MouseEvent) => {
        if (props.setPartHighlight) {
            props.setPartHighlight(part);
        }
    };
    const onPartLeave = (part: Id<wasm.Part>, event: React.MouseEvent) => {
        if (props.setPartHighlight) {
            props.setPartHighlight(undefined);
        }
    };

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
        parts.push(
            <li
                key={id}
                onMouseEnter={(ev) => onPartEnter(id, ev)}
                onMouseLeave={(ev) => onPartLeave(id, ev)}
                className={id == props.partHighlight ? "partHighlight" : ""}
                >
                {part.name}<br/>
                HP: {part.curHp}/{part.maxHp}
            </li>
        );
    }
    return (<div className="uibox">
        <div style={apStyle}>AP: {props.creature.curAp + apDelta}</div>
        <div style={mpStyle}>MP: {props.creature.curMp + mpDelta}</div>
        <ul>{parts}</ul>
    </div>);
}

export function CreatureIntent(props: {
    npc: wasm.NPC,
    coords: DOMPointReadOnly,
}): JSX.Element {
    const height = window.innerHeight;
    const style = {
        left: props.coords.x,
        bottom: height - props.coords.y,
    };
    return (<div className="intent" style={style}>
        {props.npc.motion}<br/>{props.npc.action}
    </div>);
}