import * as React from "react";

import {Preview} from "../ts/stack/preview";
import {GameOverState} from "../ts/states/game_over";
import {LevelState} from "../ts/states/level";
import {UpdateState} from "../ts/states/update";
import * as wasm from "../wasm";
import {CreatureStats, CreatureIntent} from "./creature";
import {FloatText} from "./float";
import {StackData} from "./index";
import {PlayerControls} from "./player";
import {TargetPart} from "./target";

export const WorldContext = React.createContext((undefined as unknown) as wasm.World);

export function Level(props: {}): JSX.Element {
    const data = React.useContext(StackData);
    const level = data.get(LevelState.Data)!;
    const world = level.world;

    const creatures = [];
    for (let creature of world.getCreatures()) {
        if (creature.id == world.playerId) { continue; }
        creatures.push(<CreatureStats key={creature.id} creature={creature}/>);
    }

    let intents: JSX.Element[] = [];
    if (!data.get(UpdateState.UI)?.isEndTurn) {
        intents = level.getIntents().map(([cid, intent, point]) =>
            <CreatureIntent key={cid} intent={intent} coords={point}></CreatureIntent>);
    }

    const floats: FloatText.ItemId[] = [];
    const prevFloats = data.get(Preview)?.float;
    if (prevFloats) { floats.push(...prevFloats); }
    floats.push(...level.floats.all);

    return (
        <WorldContext.Provider value={world}>
            <div className="topleft"><PlayerControls/></div>
            <div className="topright">{creatures}</div>
            {intents}
            {floats.map(ft => <FloatText key={ft.id} item={ft}></FloatText>)}
            <TargetPart/>
            <GameOver/>
        </WorldContext.Provider>
    );
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