import * as React from "react";

import {Preview} from "../ts/preview";
import {Stack} from "../ts/stack";
import {GameOverState} from "../ts/states/game_over";
import {UpdateState} from "../ts/states/update";
import * as wasm from "../wasm";
import {CreatureStats, CreatureIntent} from "./creature";
import {FloatText} from "./float";
import {PlayerControls} from "./player";
import {TargetPart} from "./target";

export const StackData = React.createContext((undefined as unknown) as Stack.DataView);
export const WorldContext = React.createContext((undefined as unknown) as wasm.World);

export function Index(props: {
    world: wasm.World,
    data: Stack.DataView,
    intents: [wasm.Creature, DOMPointReadOnly][],
}): JSX.Element {
    const world = props.world;

    const creatures = [];
    for (let id of world.getCreatureIds()) {
        if (id == world.playerId) { continue; }
        creatures.push(<CreatureStats key={id} creature={world.getCreature(id)!}/>);
    }

    let intents: JSX.Element[] = [];
    if (!props.data.get(UpdateState.UI)?.isEndTurn) {
        intents = props.intents.map(([creature, point]) =>
            <CreatureIntent key={creature.id} creature={creature} coords={point}></CreatureIntent>);
    }

    const floats: FloatText.ItemId[] = [];
    const prevFloats = props.data.get(Preview)?.float;
    if (prevFloats) { floats.push(...prevFloats); }
    const updFloats = props.data.get(UpdateState.UI)?.float.all;
    if (updFloats) { floats.push(...updFloats); }

    return (
    <StackData.Provider value={props.data}>
        <WorldContext.Provider value={props.world}>
            <div className="topleft"><PlayerControls/></div>
            <div className="topright">{creatures}</div>
            {intents}
            {floats.map(ft => <FloatText key={ft.id} item={ft}></FloatText>)}
            <TargetPart/>
            <GameOver/>
        </WorldContext.Provider>
    </StackData.Provider>);
}

function GameOver(props: {}): JSX.Element | null {
    const data = React.useContext(StackData);
    const state = data.get(GameOverState.UI)?.state;
    if (!state) { return null; }
    let text: string;
    switch (state) {
        case "Lost": text = "You Lost!"; break;
        case "Won": text = "You Won!"; break;
        default: text = `ERROR: ${state}`;
    }
    return <div className="gameOver uibox">{text}</div>;
}