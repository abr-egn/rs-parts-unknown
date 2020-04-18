import * as React from "react";

import * as wasm from "../wasm";

import * as states from "../ts/states";

export function CardList(props: {
    active: boolean,
    cards: wasm.Card[],
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
  
    props.cards.sort((a, b) => {
        if (a.creatureId != b.creatureId) {
            return a.creatureId - b.creatureId;
        }
        if (a.partId != b.partId) {
            return a.partId - b.partId;
        }
        return a.id - b.id;
    });
  
    const list = props.cards.map((card) =>
        <li key={cardKey(card)}>
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