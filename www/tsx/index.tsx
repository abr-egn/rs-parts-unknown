import * as React from "react";

import {Stack} from "../ts/stack";
import {LevelState} from "../ts/states/level";
import {Level} from "./level";

export const StackData = React.createContext((undefined as unknown) as Stack.DataView);

export function Index(props: {
    data: Stack.DataView,
}): JSX.Element {
    let level: JSX.Element | undefined;
    if (props.data.get(LevelState.Data)) {
        level = <Level></Level>;
    }

    return (
        <StackData.Provider value={props.data}>
            {level}
        </StackData.Provider>
    );
}