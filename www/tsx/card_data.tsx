import * as React from "react";

export type CardText = (props: {ui: any}) => JSX.Element;

export interface CardDisplay {
    icon: string,
    text: CardText,
}

export type CardData = { [key: string]: CardDisplay };

export const CARDS: Readonly<CardData> = Object.freeze({
    "Block": {
        icon: "shield.svg",
        text: () => (<span>
            <Expose/> this part to <Guard/> another until your next turn.
        </span>),
    },
    "Punch": {
        icon: "punch.svg",
        text: (props) => (<span>
            <span className="tag">Hit</span> an adjacent enemy for
            {' '}<Scaled data={props.ui.damage}/> <Tags tags={props.ui.tags}/>
            {' '}damage.
        </span>),
    },
    "Rage": {
        icon: "angry-eyes.svg",
        text: (props) => (<span>
            Add <Scaled data={props.ui.added}/> damage to your
            {' '}<span className="tag">Hit</span>s until end of turn.
        </span>),
    },
    "Regenerate": {
        icon: "healing.svg",
        text: (props) => (<span>
            <Heal/> a damaged <span className="tag">Flesh</span> part for 
            {' '}<Scaled data={props.ui.heal}/> HP.
        </span>),
    },
    "Stagger": {
        icon: "foot-trip.svg",
        text: () => (<span>
            <Expose/> a random part on an adjacent enemy until end of turn.
        </span>),
    },
    "Throw Debris": {
        icon: "thrown-charcoal.svg",
        text: (props) => (<span>
            <span className="tag">Hit</span> a visible enemy for 
            {' '}<Scaled data={props.ui.damage}/> <Tags tags={props.ui.tags}/>
            {' '}damage.
        </span>),
    },
});

function Expose(): JSX.Element {
    return <div className="keyword">
        Expose
        <span className="tooltip">
            The part gains <span className="tag">Open</span>, making it
            targetable by attacks.
        </span>
    </div>;
}

function Guard(): JSX.Element {
    return <div className="keyword">
        Guard
        <span className="tooltip">
            The part loses <span className="tag">Open</span>, making it
            untargetable by most attacks.
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

function Scaled(props: {
    data: {delta: string, value: number},
}): JSX.Element {
    const classes = ["scaled", props.data.delta];
    return <span className={classes.join(" ")}>{props.data.value}</span>
}

function Tags(props: {
    tags: string[],
}): JSX.Element {
    const tags = props.tags.map(tag => <b>{tag}</b>);
    return <>{tags}</>;
}