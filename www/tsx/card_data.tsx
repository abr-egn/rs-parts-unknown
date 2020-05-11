import * as React from "react";

export type CardText = (props: {}) => JSX.Element;

export interface CardDisplay {
    icon: string,
    text: CardText,
}

export type CardData = { [key: string]: CardDisplay };

export const CARDS: Readonly<CardData> = Object.freeze({
    "Punch": {
        icon: "unimplemented",
        text: (props) => <p>Hello World</p>,
    },
});