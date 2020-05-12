import * as React from "react";

export type CardText = (props: {ui: Map<string, string>}) => JSX.Element;

export interface CardDisplay {
    icon: string,
    text: CardText,
}

export type CardData = { [key: string]: CardDisplay };

export const CARDS: Readonly<CardData> = Object.freeze({
    "Guard": {
        icon: "shield.svg",
        text: () => <span><Expose/> this part to <Guard/> another until your next turn.</span>
    },
    "Heal": {
        icon: "healing.svg",
        text: () => <span>Heal a damaged part for 5 HP.</span>
    },
    "Punch": {
        icon: "punch.svg",
        text: (props) => <span>Hit an adjacent enemy for {props.ui.get("damage")} damage.</span>,
    },
    "Rage": {
        icon: "angry-eyes.svg",
        text: () => <span>Add 7 damage to your hits until end of turn.</span>
    },
    "Stagger": {
        icon: "foot-trip.svg",
        text: () => <span><Expose/> a random part on an adjacent enemy until end of turn.</span>
    },
    "Throw Debris": {
        icon: "thrown-charcoal.svg",
        text: (props) => <span>Hit a visible enemy for {props.ui.get("damage")} damage.</span>
    },
});

function Expose(): JSX.Element {
    return (<>
        <div className="keyword">
            Expose
            <span className="tooltip">
                Remove <span className="keyword">Guard</span> from the part, making
                it targetable by attacks.
            </span>
        </div>
    </>);
}

function Guard(): JSX.Element {
    return <div className="keyword">
        Guard
        <span className="tooltip">
            Prevent this part from being targeted by attacks.
        </span>
    </div>;
}