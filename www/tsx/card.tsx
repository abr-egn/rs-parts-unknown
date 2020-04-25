import * as React from "react";

import * as wasm from "../wasm";
import {Id} from "../wasm";

import {PlayCardState} from "../ts/states/play_card";

import {WorldContext} from "./index";

export function CardList(props: {
    active: boolean,
    cards: wasm.Card[],
    partHighlight?: Id<wasm.Part>,
    setPartHighlight: (part: Id<wasm.Part> | undefined) => void,
}): JSX.Element {
    const world = React.useContext(WorldContext);

    const startPlay = (creatureId: Id<wasm.Creature>, ix: number) => {
        window.game.stack.push(new PlayCardState(creatureId, ix));
    };
    const canPlay = (card: wasm.Card): boolean => {
        return world.checkSpendAP(card.creatureId, card.apCost);
    };
    const cardKey = (card: wasm.Card): string => {
        return `(${card.creatureId},${card.partId},${card.id})`;
    };
    const onCardEnter = (card: wasm.Card, event: React.MouseEvent) => {
        props.setPartHighlight(card.partId);
    };
    const onCardLeave = (card: wasm.Card, event: React.MouseEvent) => {
        props.setPartHighlight(undefined);
    }
  
    const list = props.cards.map((card, ix) => {
        const playable = props.active && canPlay(card);
        let onClick = undefined;
        const classes = [];
        if (playable) {
            classes.push("validTarget");
            onClick = () => startPlay(card.creatureId, ix);
        } else {
            classes.push("invalidTarget");
        }
        if (card.partId == props.partHighlight) {
            classes.push("partHighlight");
        }
        return (
            <li key={cardKey(card)}
                onMouseEnter={(ev) => onCardEnter(card, ev)}
                onMouseLeave={(ev) => onCardLeave(card, ev)}
                onMouseDown={onClick}
                className={classes.join(" ")}
                >
                [{card.apCost}] {card.name}
            </li>
        );
    });
    return <ul>{list}</ul>;
  }