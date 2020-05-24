import * as React from "react";

import {Stack} from "../ts/stack";

import {EndTurnState} from "../ts/states/end_turn";
import {LevelState} from "../ts/states/level";
import {PlayCardState} from "../ts/states/play_card";
import {MovePlayerState} from "../ts/states/move_player";

import {Hand} from "./card";
import {CreatureStats} from "./creature";
import {StackData} from "./index";
import {WorldContext} from "./level";
import {RootPortal} from "./root";

export function PlayerControls(props: {}): JSX.Element {
    const world = React.useContext(WorldContext);
    const player = world.getCreature(world.playerId)!;

    const data = React.useContext(StackData);
    const active = data.get(Stack.Active);
    const play = data.get(PlayCardState.UI);
    const toUpdate = data.get(PlayCardState.ToUpdate);

    const movePlayer = () => window.game.stack.push(new MovePlayerState());

    const hasAp = player.curAp > 0;
    const hasMp = player.curMp > 0;

    const baseActive = Boolean(active?.is(LevelState));
    const inPlay = Boolean(active?.is(PlayCardState) && !toUpdate);

    return (<div>
        <CreatureStats
            creature={player}
        />
        <RootPortal>
            <div className="bottomleft">
                <div className="card pile">
                    <div>Draw</div>
                    {player.draw.length}
                </div>
            </div>
            <div className="bottom">
                <Hand
                    active={hasAp && baseActive}
                    cards={player.hand}
                    playing={play?.card}
                />
            </div>
            <div className="bottomright">
                <div className="card pile">
                    <div>Discard</div>
                    {player.discard.length}
                </div>
            </div>
        </RootPortal>
        {inPlay && <div>Playing: {play?.card.name}</div>}
        {baseActive && <EndTurnButton/>}
        {baseActive && hasMp && <button onClick={movePlayer}>Move</button>}
    </div>);
}

function EndTurnButton(props: {}): JSX.Element {
    const onClick = () => window.game.stack.push(new EndTurnState());
    return <button onClick={onClick}>End Turn</button>;
}