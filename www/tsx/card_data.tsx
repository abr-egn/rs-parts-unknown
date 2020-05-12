import * as React from "react";

export type CardText = (props: {ui: Map<string, string>}) => JSX.Element;

export interface CardDisplay {
    icon: string,
    text: CardText,
}

export type CardData = { [key: string]: CardDisplay };

export const CARDS: Readonly<CardData> = Object.freeze({
    "Block": {
        icon: "shield.svg",
        text: () => <span><Expose/> this part to <Guard/> another until your next turn.</span>
    },
    "Punch": {
        icon: "punch.svg",
        text: (props) => <span><span className="tag">Hit</span> an adjacent enemy for {props.ui.get("damage")} damage.</span>,
    },
    "Rage": {
        icon: "angry-eyes.svg",
        text: () => <span>Add 7 damage to your <span className="tag">Hit</span>s until end of turn.</span>
    },
    "Regenerate": {
        icon: "healing.svg",
        text: () => <span><Heal/> a damaged <span className="tag">Flesh</span> part for 5 HP.</span>
    },
    "Stagger": {
        icon: "foot-trip.svg",
        text: () => <span><Expose/> a random part on an adjacent enemy until end of turn.</span>
    },
    "Throw Debris": {
        icon: "thrown-charcoal.svg",
        text: (props) => <span><span className="tag">Hit</span> a visible enemy for {props.ui.get("damage")} damage.</span>
    },
});

function Expose(): JSX.Element {
    return <div className="keyword">
        Expose
        <span className="tooltip">
            Remove <span className="keyword">Guard</span> from the part, making
            it targetable by attacks.
        </span>
    </div>;
}

function Guard(): JSX.Element {
    return <div className="keyword">
        Guard
        <span className="tooltip">
            Prevent this part from being targeted by attacks.
        </span>
    </div>;
}

function Heal(): JSX.Element {
    return <div className="keyword">
        Heal
        <span className="tooltip">
            Restore hitpoints, and remove <span className="keyword">Broken</span>.
        </span>
    </div>;
}