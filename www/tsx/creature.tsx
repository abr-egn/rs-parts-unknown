import * as React from "react";

import {Focus} from "../ts/stack/focus";
import {Highlight} from "../ts/stack/highlight";
import {Preview} from "../ts/stack/preview";
import * as wasm from "../wasm";
import {StackData} from "./index";
import {WorldContext} from "./level";

export function CreatureStats(props: {
    creature: wasm.Creature,
}): JSX.Element {
    const data = React.useContext(StackData);
    const stats = data.get(Preview)?.stats.get(props.creature.id);
    const focus = data.get(Focus);
    const highlight = data.get(Highlight);

    let sorted = Array.from(props.creature.parts.values());
    sorted.sort((a, b) => a.id - b.id);
    let parts = [];
    for (let part of sorted) {
        parts.push(<PartStats part={part}/>);
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

    let className = "uibox";
    if (highlight?.static.creatures.has(props.creature.id)) {
        className = className + " highlight";
    }
    if (highlight?.throb.creatures.has(props.creature.id)) {
        className = className + " throb";
    }
    return (
    <div
        className={className}
        onMouseEnter={() => focus?.creature?.onEnter?.(props.creature.id)}
        onMouseLeave={() => focus?.creature?.onLeave?.(props.creature.id)}
        onMouseDown={() => focus?.creature?.onClick?.(props.creature.id)}
    >
        <div>{props.creature.name}</div>
        <div style={apStyle}>AP: {props.creature.curAp + apDelta}</div>
        <div style={mpStyle}>MP: {props.creature.curMp + mpDelta}</div>
        {parts}
    </div>);
}

function PartStats(props: {
    part: wasm.Part,
}): JSX.Element {
    const data = React.useContext(StackData);
    const focus = data.get(Focus);
    const highlight = data.get(Highlight);
    const stats = data.get(Preview)?.stats.get(props.part.creatureId);

    const classNames = ["part"];
    /*
    if (part.tags.includes("Open")) {
        classNames.push("partOpen");
    }
    */
    if (highlight?.static.parts.get(props.part.creatureId)?.has(props.part.id)) {
        classNames.push("highlight");
    }
    if (highlight?.throb.parts.get(props.part.creatureId)?.has(props.part.id)) {
        classNames.push("throb");
    }
    const hpDelta = stats?.hpDelta.get(props.part.id) || 0;
    const hpStyle: React.CSSProperties = {};
    if (hpDelta < 0) {
        hpStyle.color = "red";
    } else if (hpDelta > 0) {
        hpStyle.color = "green";
    }

    return <div
        key={props.part.id}
        onMouseEnter={() => focus?.part.onEnter?.([props.part.creatureId, props.part.id])}
        onMouseLeave={() => focus?.part.onLeave?.([props.part.creatureId, props.part.id])}
        onMouseDown={() => focus?.part.onClick?.([props.part.creatureId, props.part.id])}
        className={classNames.join(" ")}
        >
        {props.part.name}<br/>
        <span style={hpStyle}>HP: {props.part.curHp + hpDelta}/{props.part.maxHp}</span>
    </div>
}

export function CreatureIntent(props: {
    intent: wasm.Intent,
    coords: DOMPointReadOnly,
}): JSX.Element {
    const height = window.innerHeight;
    const style = {
        left: props.coords.x,
        bottom: height - props.coords.y,
    };
    let intent: JSX.Element = <span>???</span>;
    let kind;
    if (kind = props.intent.kind.Attack) {
        let intentIcon;
        switch (kind.range) {
            case "Melee": intentIcon = "icons/punch.svg";
        }
        intent = <span><img src={intentIcon} className="attackIcon"></img>{kind.damage}</span>
    }
    return (<div className="intent" style={style}>{intent}</div>);
}