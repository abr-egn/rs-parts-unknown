import * as React from "react";

import {Focus} from "../ts/focus";
import {Highlight} from "../ts/highlight";
import {Preview} from "../ts/preview";
import * as wasm from "../wasm";
import {StackData, WorldContext} from "./index";

/*
declare const phantom: unique symbol;
type Foo<T> = number & { [phantom]?: T }
function bar(f: Foo<number>): void {}
function baz(f: Foo<string>): void { bar(f) }
*/

export function CreatureStats(props: {
    creature: wasm.Creature,
}): JSX.Element {
    const data = React.useContext(StackData);
    const stats = data.get(Preview)?.stats.get(props.creature.id);
    const focus = data.get(Focus);

    let sorted = Array.from(props.creature.parts.values());
    sorted.sort((a, b) => a.id - b.id);
    let parts = [];
    for (let part of sorted) {
        const highlight = data.get(Highlight)?.parts.get(part.creatureId)?.has(part.id);
        let classNames = [];
        if (part.tags.includes("Open")) {
            classNames.push("partOpen");
        }
        if (highlight) {
            classNames.push("partHighlight");
        }
        let hpDelta = stats?.hpDelta.get(part.id) || 0;
        const hpStyle: React.CSSProperties = {};
        if (hpDelta < 0) {
            hpStyle.color = "red";
        } else if (hpDelta > 0) {
            hpStyle.color = "green";
        }
        parts.push(
            <li
                key={part.id}
                onMouseEnter={() => focus?.part.onEnter?.([part.creatureId, part.id])}
                onMouseLeave={() => focus?.part.onLeave?.([part.creatureId, part.id])}
                onMouseDown={() => focus?.part.onClick?.([part.creatureId, part.id])}
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

    const highlight = data.get(Highlight)?.creatures.has(props.creature.id);
    return (
    <div
        className={highlight?"highlightBox":"uibox"}
        onMouseEnter={() => focus?.creature?.onEnter?.(props.creature.id)}
        onMouseLeave={() => focus?.creature?.onLeave?.(props.creature.id)}
        onMouseDown={() => focus?.creature?.onClick?.(props.creature.id)}
    >
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
    if (!npc.intent) {
        intentStr = "";
    } else if (intent = npc.intent?.kind.Attack) {
        let damage_from = world.scaleDamageFrom(intent.base_damage, props.creature.id, npc.intent.from);
        let damage = world.scaleDamageTo(damage_from, world.playerId, undefined);
        intentStr = `${intent.range} Attack: ${damage}`;
    }
    return (<div className="intent" style={style}>
        {npc.motion}<br/>{intentStr}
    </div>);
}