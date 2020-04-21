import * as React from "react";

import * as wasm from "../wasm";
import {Id} from "../wasm";

import {StatPreview} from "../ts/highlight";
import {Stack} from "../ts/stack";
import * as states from "../ts/states";

import {CardList} from "./card";
import {CreatureStats} from "./creature";

export function PlayerControls(props: {
    player: wasm.Creature,
    active?: Stack.Active,
    play?: states.PlayCard.UI,
    stats?: StatPreview,
}): JSX.Element {
    const cancelPlay = () => window.game.stack.pop();
    const movePlayer = () => window.game.stack.push(new states.MovePlayer());
    // Force the type checker to type it this way.
    const defPartHi = (): Id<wasm.Part> | undefined => { return undefined; }
    const [partHighlight, setPartHighlight] = React.useState(defPartHi());

    const canPlay = props.active?.is(states.Base) || false;
    const inPlay = props.active?.is(states.PlayCard) || false;
    const canCancel = (inPlay || props.active?.is(states.MovePlayer)) || false;

    return (<div>
        <CreatureStats
            creature={props.player}
            focused={false}
            stats={props.stats}
            partHighlight={partHighlight}
            setPartHighlight={setPartHighlight}
        />
        <CardList
            active={canPlay}
            hand={props.player.hand}
            partHighlight={partHighlight}
            setPartHighlight={setPartHighlight}
        />
        {inPlay && <div>Playing: {props.play?.card.name}</div>}
        <EndTurnButton active={canPlay}/>
        {canPlay && <button onClick={movePlayer}>Move</button>}
        {canCancel &&  <div><button onClick={cancelPlay}>Cancel</button></div>}
    </div>);
}

function EndTurnButton(props: {active: boolean}): JSX.Element {
    const onClick = () => window.game.stack.push(new states.EndTurn());
    return <button onClick={onClick} disabled={!props.active}>End Turn</button>;
}