import * as React from "react";

import * as wasm from "../wasm";
import {Id} from "../wasm";

import * as states from "../ts/states";

export function CardList(props: {
    active: boolean,
    cards: wasm.Card[],
    partHighlight?: Id<wasm.Part>,
    setPartHighlight: (part: Id<wasm.Part> | undefined) => void,
}): JSX.Element {
    const startPlay = (card: wasm.Card) => {
        window.game.stack.push(new states.PlayCard(card));
    };
    const canPlay = (card: wasm.Card): boolean => {
        return window.game.world.checkSpendAP(card.creatureId, card.apCost);
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
  
    props.cards.sort((a, b) => {
        if (a.creatureId != b.creatureId) {
            return a.creatureId - b.creatureId;
        }
        if (a.partId != b.partId) {
            return a.partId - b.partId;
        }
        return a.id - b.id;
    });
  
    // TODO: highlight source part on mouseover
    const list = props.cards.map((card) =>
        <li key={cardKey(card)}
            onMouseEnter={(ev) => onCardEnter(card, ev)}
            onMouseLeave={(ev) => onCardLeave(card, ev)}
            className={card.partId == props.partHighlight ? "partHighlight" : ""}
            >
            <button
                onClick={() => startPlay(card)}
                disabled={!props.active || !canPlay(card)}>
                Play
            </button>
            [{card.apCost}] {card.name}
        </li>
    );
    return (<div>
        Cards:
        <ul>{list}</ul>
    </div>);
  }