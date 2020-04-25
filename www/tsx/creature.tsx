import {produce} from "immer";
import * as React from "react";

import {creatureToTarget, partToTarget} from "../ts/extra";
import {Preview} from "../ts/preview";
import {Stack} from "../ts/stack";
import {BaseState} from "../ts/states/base";
import {PlayCardState} from "../ts/states/play_card";
import * as wasm from "../wasm";
import {Id} from "../wasm";
import {StackData, WorldContext} from "./index";


export function CreatureStats(props: {
    creature: wasm.Creature,
    partHighlight?: Id<wasm.Part>,
    setPartHighlight?: (part: Id<wasm.Part> | undefined) => void,
}): JSX.Element {
    const world = React.useContext(WorldContext);
    const data = React.useContext(StackData);
    const stats = data.get(Preview)?.stats.get(props.creature.id);
    const base = data.get(BaseState.UI);
    const [hovered, setHovered] = React.useState(false);
    const [partHovered, setPartHovered] = React.useState(new Set<Id<wasm.Part>>());
    const focused = Boolean(base?.hovered.has(props.creature.id)) || hovered;

    let canTarget = false;
    let partTarget: Map<Id<wasm.Part>, wasm.Target> = new Map();
    const playState = data.get(PlayCardState.UI);
    const inPlay = playState?.inPlay;
    if (data.get(Stack.Active)?.is(PlayCardState) && inPlay) {
        const target = creatureToTarget(props.creature);
        canTarget = inPlay.targetValid(world, target);
        for (let part of props.creature.parts.values()) {
            const target = partToTarget(part);
            if (inPlay.targetValid(world, target)) {
                partTarget.set(part.id, target);
            }
        }
    }
    
    const onCreatureEnter = () => {
        if (canTarget) { setHovered(true); }
    };
    const onCreatureLeave = () => {
        if (canTarget) { setHovered(false); }
    };

    const onPartEnter = (part: wasm.Part) => {
        if (props.setPartHighlight) {
            props.setPartHighlight(part.id);
        }
        if (partTarget.get(part.id)) {
            setPartHovered(produce(partHovered, (draft) => {
                draft.add(part.id);
            }));
        }
    };
    const onPartLeave = (part: wasm.Part) => {
        if (props.setPartHighlight) {
            props.setPartHighlight(undefined);
        }
        if (partTarget.get(part.id)) {
            setPartHovered(produce(partHovered, (draft) => {
                draft.delete(part.id);
            }));
        }
    };
    const onPartClick = (part: wasm.Part) => {
        const target = partTarget.get(part.id);
        if (target && playState) {
            playState.playOnTarget(target);
        }
    }

    let sorted = Array.from(props.creature.parts.values());
    sorted.sort((a, b) => a.id - b.id);
    let parts = [];
    for (let part of sorted) {
        let classNames = [];
        if (part.id == props.partHighlight) {
            classNames.push("partHighlight");
        }
        if (part.tags.includes("Open")) {
            classNames.push("open");
        }
        if (partHovered.has(part.id)) {
            classNames.push("partFocused");
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
                onMouseEnter={() => onPartEnter(part)}
                onMouseLeave={() => onPartLeave(part)}
                onMouseDown={() => onPartClick(part)}
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

    return (
    <div
        className={focused?"focusedBox":"uibox"}
        onMouseEnter={onCreatureEnter}
        onMouseLeave={onCreatureLeave}
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