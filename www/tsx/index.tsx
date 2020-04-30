import * as React from "react";

import {Stack} from "../ts/stack";
import {Preview} from "../ts/stack/preview";
import {GameOverState} from "../ts/states/game_over";
import {LevelState} from "../ts/states/level";
import {UpdateState} from "../ts/states/update";
import * as wasm from "../wasm";
import {CreatureStats, CreatureIntent} from "./creature";
import {FloatText} from "./float";
import {PlayerControls} from "./player";
import {TargetPart} from "./target";

export const StackData = React.createContext((undefined as unknown) as Stack.DataView);
export const WorldContext = React.createContext((undefined as unknown) as wasm.World);

export function Index(props: {
    data: Stack.DataView,
}): JSX.Element {
    // TODO: don't assume LevelState
    const level = props.data.get(LevelState.Data)!;
    const world = level.world;

    const creatures = [];
    for (let creature of world.getCreatures()) {
        if (creature.id == world.playerId) { continue; }
        creatures.push(<CreatureStats key={creature.id} creature={creature}/>);
    }

    let intents: JSX.Element[] = [];
    if (!props.data.get(UpdateState.UI)?.isEndTurn) {
        intents = level.getIntents().map(([creature, point]) =>
            <CreatureIntent key={creature.id} creature={creature} coords={point}></CreatureIntent>);
    }

    const floats: FloatText.ItemId[] = [];
    const prevFloats = props.data.get(Preview)?.float;
    if (prevFloats) { floats.push(...prevFloats); }
    floats.push(...level.floats.all);

    return (
    <StackData.Provider value={props.data}>
        <WorldContext.Provider value={world}>
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