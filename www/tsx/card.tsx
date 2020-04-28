import * as React from "react";

import {Focus} from "../ts/stack/focus";
import {Highlight} from "../ts/stack/highlight";
import {PlayCardState} from "../ts/states/play_card";
import * as wasm from "../wasm";
import {Id} from "../wasm";
import {StackData, WorldContext} from "./index";

export function CardList(props: {
    active: boolean,
    cards: wasm.Card[],
}): JSX.Element {
    const world = React.useContext(WorldContext);
    const data = React.useContext(StackData);
    const focus = data.get(Focus);
    const highlight = data.get(Highlight);

    const startPlay = (creatureId: Id<wasm.Creature>, ix: number) => {
        window.game.stack.push(new PlayCardState(creatureId, ix));
    };
    const canPlay = (card: wasm.Card): boolean => {
        // TODO: check part
        return world.checkSpendAP(card.creatureId, card.apCost);
    };
    const cardKey = (card: wasm.Card): string => {
        return `(${card.creatureId},${card.partId},${card.id})`;
    };
    const onCardEnter = (card: wasm.Card, event: React.MouseEvent) => {
        focus?.part?.onEnter?.([card.creatureId, card.partId]);
    };
    const onCardLeave = (card: wasm.Card, event: React.MouseEvent) => {
        focus?.part?.onLeave?.([card.creatureId, card.partId]);
    }
  
    const list = props.cards.map((card, ix) => {
        const playable = props.active && canPlay(card);
        let onClick = undefined;
        const classes = ["card"];
        if (playable) {
            classes.push("playable");
            onClick = () => startPlay(card.creatureId, ix);
            if (highlight?.static.parts.get(card.creatureId)?.has(card.partId)) {
                classes.push("highlight");
            }
        } else {
            classes.push("unplayable");
            if (highlight?.throb.parts.get(card.creatureId)?.has(card.partId)) {
                classes.push("highlight");
            }
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