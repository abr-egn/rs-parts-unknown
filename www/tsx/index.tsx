import * as React from "react";

import {Stack} from "../ts/stack";
import {LevelState} from "../ts/states/level";
import {TitleState} from "../ts/states/title";
import {Level} from "./level";

export const StackData = React.createContext((undefined as unknown) as Stack.DataView);

export function Index(props: {
    data: Stack.DataView,
}): JSX.Element {
    let content: JSX.Element;

    if (props.data.get(TitleState.UI)) {
        content = <Title></Title>;
    } else if (props.data.get(LevelState.Data)) {
        content = <Level></Level>;
    } else {
        content = <div>Error: no state</div>
    }

    return (
        <StackData.Provider value={props.data}>
            {content}
        </StackData.Provider>
    );
}

export function Title(props: {}): JSX.Element {
    const stack = React.useContext(StackData);
    const ui = stack.get(TitleState.UI)!;
    let letters: JSX.Element[] = [];
    const DELAY_DEC = 0.3;
    const TEXT = "Parts Unknown";
    let delay = DELAY_DEC * TEXT.length;
    for (let char of "Parts Unknown") {
        let style = {
            animationDelay: `${delay}s`
        };
        if (char == " ") {
            letters.push(<span className="text" style={style}>&nbsp;</span>);
        } else {
            letters.push(<span className="text" style={style}>{char}</span>);
        }
        delay -= DELAY_DEC;
    }
    return (<div className="title">
        <div className="letters">{letters}</div>
        <br/>
        <br/>
        <button onClick={ui.done}>Play</button>
    </div>);
}