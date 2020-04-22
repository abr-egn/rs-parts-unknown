import * as React from "react";

import * as wasm from "../wasm";
import {Id} from "../wasm";

import {Stack} from "../ts/stack";
import * as states from "../ts/states";

import {StackData} from "./index";
import {CardList} from "./card";
import {CreatureStats} from "./creature";

export function PlayerControls(props: {
    player: wasm.Creature,
}): JSX.Element {
    const data = React.useContext(StackData);
    const active = data.get(Stack.Active);
    const play = data.get(states.PlayCard.UI);
    const toUpdate = data.get(states.PlayCard.ToUpdate);

    const cancelPlay = () => window.game.stack.pop();
    const movePlayer = () => window.game.stack.push(new states.MovePlayer());
    const [partHighlight, setPartHighlight] = React.useState(undefined as (undefined | (() => Id<wasm.Part>)))

    const hasAp = props.player.curAp > 0;
    const hasMp = props.player.curMp > 0;

    const canPlay = Boolean(hasAp && active?.is(states.Base));
    const inPlay = Boolean(active?.is(states.PlayCard) && !toUpdate);
    const canCancel = Boolean(inPlay || active?.is(states.MovePlayer));

    return (<div>
        <CreatureStats
            creature={props.player}
            focused={false}
            partHighlight={partHighlight}
            setPartHighlight={setPartHighlight}
        />
        <CardList
            active={canPlay}
            hand={props.player.hand}
            partHighlight={partHighlight}
            setPartHighlight={setPartHighlight}
        />
        {inPlay && <div>Playing: {play?.card.name}</div>}
        {canPlay && <EndTurnButton/>}
        {canPlay && hasMp && <button onClick={movePlayer}>Move</button>}
        {canCancel &&  <div><button onClick={cancelPlay}>Cancel</button></div>}
    </div>);
}

function EndTurnButton(props: {}): JSX.Element {
    const onClick = () => window.game.stack.push(new states.EndTurn());
    return <button onClick={onClick}>End Turn</button>;
}