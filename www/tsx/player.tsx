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

    const movePlayer = () => window.game.stack.push(new MovePlayerState());

    const hasAp = player.curAp > 0;
    const hasMp = player.curMp > 0;

    const baseActive = Boolean(active?.is(LevelState));

    const pileClasses = ["card pile"];
    if (play != undefined) {
        pileClasses.push("unplayable");
    }

    return (<div>
        <CreatureStats
            creature={player}
        />
        <RootPortal>
            <div className="bottomleft player">
                <div className={pileClasses.join(" ")}>
                    <div>Draw</div>
                    {player.draw.length}
                </div>
                {baseActive && hasMp && <button onClick={movePlayer}>Move</button>}
            </div>
            <div className="bottom">
                <Hand
                    active={hasAp && baseActive}
                    cards={player.hand}
                    playing={play?.card}
                />
            </div>
            <div className="bottomright player">
                {baseActive && <EndTurnButton/>}
                <div className={pileClasses.join(" ")}>
                    <div>Discard</div>
                    {player.discard.length}
                </div>
            </div>
        </RootPortal>
    </div>);
}

function EndTurnButton(props: {}): JSX.Element {
    const onClick = () => window.game.stack.push(new EndTurnState());
    return <button onClick={onClick}>End Turn</button>;
}