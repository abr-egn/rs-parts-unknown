import * as React from "react";

import * as wasm from "../wasm";

import * as states from "../ts/states";

export function TargetPart(props: {
    target: states.TargetPart.UI,
}): JSX.Element {
    const point = window.game.hexCoords(props.target.hex);
    const style = {
        left: point.x,
        top: point.y,
    };
    let parts = props.target.targets.map(([part, valid]) =>
        <PartMenuItem
            key={part.id}
            part={part}
            valid={valid}
            callbacks={props.target.callbacks}
        ></PartMenuItem>);
    return <div className="partTargetMenu" style={style}>{parts}</div>;
}

export function PartMenuItem(props: {
    part: wasm.Part,
    valid: boolean,
    callbacks: states.TargetPart.Callbacks,
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