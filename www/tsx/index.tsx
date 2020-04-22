import * as React from "react";

import * as wasm from "../wasm";
import {Id} from "../wasm";

import {Highlight} from "../ts/highlight";
import {Stack} from "../ts/stack";
import * as states from "../ts/states";

import {CreatureStats, CreatureIntent} from "./creature";
import {PlayerControls} from "./player";
import {TargetPart} from "./target";

export const StackData = React.createContext((undefined as unknown) as Stack.DataView);
export const WorldContext = React.createContext((undefined as unknown) as wasm.World);

export function Index(props: {
    world: wasm.World,
    data: Stack.DataView,
    intents: [Id<wasm.Creature>, wasm.NPC, DOMPointReadOnly][],
}): JSX.Element {
    const world = props.world;
    const base = props.data.get(states.Base.UI);

    let creatures = [];
    for (let id of world.getCreatureIds()) {
        if (id == world.playerId) { continue; }
        const creature = world.getCreature(id);
        if (creature) {
            creatures.push(<CreatureStats
                key={id}
                creature={creature}
                focused={base?.hovered.has(id) || false}
            />);
        }
    }
    const gameOverState = props.data.get(states.GameOver.UI)?.state;
    let gameOver = undefined;
    if (gameOverState) {
        gameOver = <GameOver state={gameOverState}/>;
    }
    let intents: JSX.Element[] = [];
    if (!props.data.get(Stack.Active)?.is(states.Update)) {
        intents = props.intents.map(([id, npc, point]) =>
            <CreatureIntent key={id} npc={npc} coords={point}></CreatureIntent>);
    }
    let targetPart = props.data.get(states.TargetPart.UI);
    return (
    <StackData.Provider value={props.data}>
    <WorldContext.Provider value={props.world}>
        <div>
            <div className="topleft">
                <PlayerControls
                    player={world.getCreature(world.playerId)!}
                />
            </div>
            <div className="topright">
                {creatures}
            </div>
            {intents}
            {targetPart ? <TargetPart target={targetPart}></TargetPart> : null}
            {gameOver}
        </div>
    </WorldContext.Provider>
    </StackData.Provider>);
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