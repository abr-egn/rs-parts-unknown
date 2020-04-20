import * as React from "react";

import * as states from "../ts/states";

export function TargetPart(props: {
    target: states.TargetPart.UI,
}): JSX.Element {
    const point = window.game.hexCoords(props.target.hex);
    const style = {
        left: point.x,
        top: point.y,
    };
    return <div className="partTarget" style={style}>Part Menu</div>;
}