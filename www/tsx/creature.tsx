import * as React from "react";

import * as wasm from "../wasm";
import {Id} from "../wasm";

import {Highlight} from "../ts/highlight";
import * as states from "../ts/states";

import {StackData, WorldContext} from "./index";

export function CreatureStats(props: {
    creature: wasm.Creature,
    partHighlight?: Id<wasm.Part>,
    setPartHighlight?: (part: Id<wasm.Part> | undefined) => void,
}): JSX.Element {
    const data = React.useContext(StackData);
    const stats = data.get(Highlight)?.stats.get(props.creature.id);
    const base = data.get(states.Base.UI);
    const focused = Boolean(base?.hovered.has(props.creature.id));

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

    let sorted = Array.from(props.creature.parts);
    sorted.sort(([id_a, _p_a], [id_b, _p_b]) => id_a - id_b);
    let parts = [];
    for (let [id, part] of sorted) {
        let classNames = [];
        if (id == props.partHighlight) {
            classNames.push("partHighlight");
        }
        if (part.tags.includes("Open")) {
            classNames.push("open");
        }
        let hpDelta = stats?.hpDelta.get(id) || 0;
        const hpStyle: React.CSSProperties = {};
        if (hpDelta < 0) {
            hpStyle.color = "red";
        } else if (hpDelta > 0) {
            hpStyle.color = "green";
        }
        parts.push(
            <li
                key={id}
                onMouseEnter={(ev) => onPartEnter(id, ev)}
                onMouseLeave={(ev) => onPartLeave(id, ev)}
                className={classNames.join(" ")}
                >
                {part.name}<br/>
                <span style={hpStyle}>HP: {part.curHp + hpDelta}/{part.maxHp}</span>
            </li>
        );
    }

    let apDelta = stats?.statDelta.get("AP") || 0;
    const apStyle: React.CSSProperties = {};
    if (apDelta < 0) {
        apStyle.color = "red";
    } else if (apDelta > 0) {
        apStyle.color = "green";
    }
    let mpDelta = stats?.statDelta.get("MP") || 0;
    const mpStyle: React.CSSProperties = {};
    if (mpDelta < 0) {
        mpStyle.color = "red";
    } else if (mpDelta > 0) {
        mpStyle.color = "green";
    }

    return (<div className={focused?"focusedBox":"uibox"}>
        <div>{props.creature.name}</div>
        <div style={apStyle}>AP: {props.creature.curAp + apDelta}</div>
        <div style={mpStyle}>MP: {props.creature.curMp + mpDelta}</div>
        <ul>{parts}</ul>
    </div>);
}

export function CreatureIntent(props: {
    creature: wasm.Creature,
    coords: DOMPointReadOnly,
}): JSX.Element {
    const world = React.useContext(WorldContext);
    const npc = props.creature.npc!;
    const height = window.innerHeight;
    const style = {
        left: props.coords.x,
        bottom: height - props.coords.y,
    };
    let intentStr = "???";
    let intent;
    if (intent = npc.intent?.kind.Attack) {
        let damage_from = world.scaleDamageFrom(intent.base_damage, props.creature.id, npc.intent.from);
        let damage = world.scaleDamageTo(damage_from, world.playerId, undefined);
        intentStr = `${intent.range} Attack: ${damage}`;
    }
    return (<div className="intent" style={style}>
        {npc.motion}<br/>{intentStr}
    </div>);
}