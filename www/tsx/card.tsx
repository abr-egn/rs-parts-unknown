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
    const world = React.useContext(WorldContext);
    const data = React.useContext(StackData);
    const focus = data.get(Focus);
    const highlight = data.get(Highlight);

    const startPlay = (creatureId: Id<wasm.Creature>, ix: number) => {
        window.game.stack.push(new PlayCardState(creatureId, ix));
    };
    const canPlay = (card: wasm.Card): boolean => {
        return world.isPlayable(card);
    };
    const cardKey = (card: wasm.Card): string => {
        return `(${card.creatureId},${card.partId},${card.id})`;
    };
    const onCardEnter = (card: wasm.Card, event: React.MouseEvent) => {
        const playable = props.active && canPlay(card);
        if (playable) {
            focus?.part?.onEnter?.([card.creatureId, card.partId]);
        }
    };
    const onCardLeave = (card: wasm.Card, event: React.MouseEvent) => {
        const playable = props.active && canPlay(card);
        if (playable) {
            focus?.part?.onLeave?.([card.creatureId, card.partId]);
        }
    }
  
    const list = props.cards.map((card, ix) => {
        let creature = world.getCreature(card.creatureId)!;
        let part = creature.parts.get(card.partId)!;
        const playable = props.active && canPlay(card);
        let onClick = undefined;
        const classes = ["card"];
        let lit = false;
        if (playable) {
            classes.push("playable");
            onClick = () => startPlay(card.creatureId, ix);
            lit = Boolean(highlight?.static.parts.get(card.creatureId)?.has(card.partId));
        } else {
            if (props.playing?.creatureId == card.creatureId && props.playing.id == card.id) {
                classes.push("playing");
            } else {
                classes.push("unplayable");
            }
            lit = Boolean(highlight?.throb.parts.get(card.creatureId)?.has(card.partId));
        }
        if (lit) {
            classes.push("lit");
        }
        return (
            <div key={cardKey(card)}
                onMouseEnter={(ev) => onCardEnter(card, ev)}
                onMouseLeave={(ev) => onCardLeave(card, ev)}
                onMouseDown={onClick}
                className={classes.join(" ")}
            >
                <div className="databar">
                    <div className="name">{card.name}</div>
                </div>
                <div className="picture"></div>
                <div className="databar">
                    <div className="cardpart">{part.name}</div>
                    <div className="cost">{card.apCost}</div>
                </div>
                <div className="cardtext">Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do...</div>
            </div>
        );
    });
    return <div className="hand">{list}</div>;
  }