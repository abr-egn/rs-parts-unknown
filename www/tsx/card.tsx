import * as React from "react";

import {Focus} from "../ts/stack/focus";
import {Highlight} from "../ts/stack/highlight";
import {PlayCardState} from "../ts/states/play_card";
import * as wasm from "../wasm";
import {Id} from "../wasm";
import {StackData} from "./index";
import {WorldContext} from "./level";

export function Hand(props: {
    active: boolean,
    cards: wasm.Card[],
    playing?: wasm.Card,
}): JSX.Element {
    const cardKey = (card: wasm.Card): string => {
        return `(${card.creatureId},${card.partId},${card.id})`;
    };
  
    const list = props.cards.map((card, ix) => {
        return (<Card
            key={cardKey(card)}
            active={props.active}
            card={card}
            ix={ix}
            playing={props.playing?.creatureId == card.creatureId
                && props.playing.partId == card.partId
                && props.playing.id == card.id}
        ></Card>);
    });
    return <div className="hand">{list}</div>;
}

export function Card(props: {
    active: boolean,
    card: wasm.Card,
    ix: number,
    playing: boolean,
}): JSX.Element {
    const world = React.useContext(WorldContext);
    const data = React.useContext(StackData);
    const focus = data.get(Focus);
    const highlight = data.get(Highlight);
    const playable = props.active && world.isPlayable(props.card);
    const creature = world.getCreature(props.card.creatureId)!;
    const part = creature.parts.get(props.card.partId)!;

    const onEnter = () => {
        if (playable) {
            focus?.part?.onEnter?.([props.card.creatureId, props.card.partId]);
        }
    };
    const onLeave = () => {
        if (playable) {
            focus?.part?.onLeave?.([props.card.creatureId, props.card.partId]);
        }
    }
    const onClick = () => {
        if (playable) {
            window.game.stack.push(new PlayCardState(props.card.creatureId, props.ix));
        }
    }

    const classes = ["card"];
    if (playable) {
        classes.push("playable");
        if (highlight?.static.parts.get(props.card.creatureId)?.has(props.card.partId)) {
            classes.push("lit");
        }
    } else {
        if (props.playing) {
            classes.push("playing");
        } else {
            classes.push("unplayable");
        }
        if (highlight?.throb.parts.get(props.card.creatureId)?.has(props.card.partId)) {
            classes.push("lit");
        }
    }

    return (
        <div
            onMouseEnter={onEnter}
            onMouseLeave={onLeave}
            onMouseDown={onClick}
            className={classes.join(" ")}
        >
            <div className="databar">
                <div className="name">{props.card.name}</div>
            </div>
            <div className="picture"></div>
            <div className="databar">
                <div className="cardpart">{part.name}</div>
                <div className="cost">{props.card.apCost}</div>
            </div>
            <div className="cardtext">Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do...</div>
        </div>
    );
}