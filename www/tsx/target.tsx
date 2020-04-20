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
            onSelect={props.target.onSelect}
        ></PartMenuItem>);
    return <div className="partTargetMenu" style={style}>{parts}</div>;
}

export function PartMenuItem(props: {
    part: wasm.Part,
    valid: boolean,
    onSelect: (part: wasm.Part) => void,
}): JSX.Element {
    return (
        <div
            className={props.valid?"validTarget":"invalidTarget"}
            onMouseDown={() => props.onSelect(props.part)}
            >
            {props.part.name}
        </div>
    );
}