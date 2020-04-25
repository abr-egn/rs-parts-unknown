import * as React from "react";

import * as wasm from "../wasm";
import {Id} from "../wasm";

import {Stack} from "../ts/stack";
import * as states from "../ts/states";

import {CardList} from "./card";
import {CreatureStats} from "./creature";
import {StackData, WorldContext} from "./index";

export function PlayerControls(props: {}): JSX.Element {
    const world = React.useContext(WorldContext);
    const player = world.getCreature(world.playerId)!;

    const data = React.useContext(StackData);
    const active = data.get(Stack.Active);
    const play = data.get(states.PlayCard.UI);
    const toUpdate = data.get(states.PlayCard.ToUpdate);

    const cancelPlay = () => window.game.stack.pop();
    const movePlayer = () => window.game.stack.push(new states.MovePlayer());
    const [partHighlight, setPartHighlight] = React.useState(undefined as (undefined | (() => Id<wasm.Part>)))

    const hasAp = player.curAp > 0;
    const hasMp = player.curMp > 0;

    const baseActive = Boolean(active?.is(states.Base));
    const inPlay = Boolean(active?.is(states.PlayCard) && !toUpdate);
    const canCancel = Boolean(inPlay || active?.is(states.MovePlayer));

    return (<div>
        <CreatureStats
            creature={player}
            partHighlight={partHighlight}
            setPartHighlight={setPartHighlight}
        />
        <div>
            Hand:
            <CardList
                active={hasAp && baseActive}
                cards={player.hand}
                partHighlight={partHighlight}
                setPartHighlight={setPartHighlight}
            />
        </div>
        <div>
            Draw:
            <CardList
                active={false}
                cards={player.draw}
                partHighlight={partHighlight}
                setPartHighlight={setPartHighlight}
            />
        </div>
        <div>
            Discard:
            <CardList
                active={false}
                cards={player.discard}
                partHighlight={partHighlight}
                setPartHighlight={setPartHighlight}
            />
        </div>
        {inPlay && <div>Playing: {play?.card.name}</div>}
        {baseActive && <EndTurnButton/>}
        {baseActive && hasMp && <button onClick={movePlayer}>Move</button>}
        {canCancel &&  <div><button onClick={cancelPlay}>Cancel</button></div>}
    </div>);
}

function EndTurnButton(props: {}): JSX.Element {
    const onClick = () => window.game.stack.push(new states.EndTurn());
    return <button onClick={onClick}>End Turn</button>;
}