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
    toUpdate?: states.PlayCard.ToUpdate,
    stats?: StatPreview,
}): JSX.Element {
    const cancelPlay = () => window.game.stack.pop();
    const movePlayer = () => window.game.stack.push(new states.MovePlayer());
    // Force the type checker to type it this way.
    const defPartHi = (): Id<wasm.Part> | undefined => { return undefined; }
    const [partHighlight, setPartHighlight] = React.useState(defPartHi());

    const hasAp = props.player.curAp > 0;
    const hasMp = props.player.curMp > 0;

    const canPlay = Boolean(hasAp && props.active?.is(states.Base));
    const inPlay = Boolean(props.active?.is(states.PlayCard) && !props.toUpdate);
    const canCancel = Boolean(inPlay || props.active?.is(states.MovePlayer));

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
        {canPlay && <EndTurnButton/>}
        {canPlay && hasMp && <button onClick={movePlayer}>Move</button>}
        {canCancel &&  <div><button onClick={cancelPlay}>Cancel</button></div>}
    </div>);
}

function EndTurnButton(props: {}): JSX.Element {
    const onClick = () => window.game.stack.push(new states.EndTurn());
    return <button onClick={onClick}>End Turn</button>;
}