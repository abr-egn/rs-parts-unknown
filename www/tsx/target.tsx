import * as React from "react";

import * as wasm from "../wasm";
import {TargetPartState} from "../ts/states/target_part";
import {StackData} from "./index";

export function TargetPart(props: {}): JSX.Element | null {
    const data = React.useContext(StackData);
    const target = data.get(TargetPartState.UI);
    if (!target) { return null; }

    const point = window.game.board.hexCoords(target.hex);
    const style = {
        left: point.x,
        top: point.y,
    };
    let parts = target.targets.map(([part, valid]) =>
        <PartMenuItem
            key={part.id}
            part={part}
            valid={valid}
            callbacks={target.callbacks}
        ></PartMenuItem>);
    return <div className="partTargetMenu" style={style}>{parts}</div>;
}

export function PartMenuItem(props: {
    part: wasm.Part,
    valid: boolean,
    callbacks: TargetPartState.Callbacks,
}): JSX.Element {
    let onMouseDown = undefined;
    let onMouseEnter = undefined;
    let onMouseLeave = undefined;
    if (props.valid) {
        onMouseDown = () => props.callbacks.onSelect(props.part);
        onMouseEnter = () => props.callbacks.onHoverEnter(props.part);
        onMouseLeave = props.callbacks.onHoverLeave;
    }
    return (
        <div
            className={props.valid?"validTarget":"invalidTarget"}
            onMouseDown={onMouseDown}
            onMouseEnter={onMouseEnter}
            onMouseLeave={onMouseLeave}
            >
            {props.part.name}
        </div>
    );
}