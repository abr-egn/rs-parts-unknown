import * as React from "react";

import * as wasm from "../wasm";
import {Focus} from "../ts/stack/focus";
import {LevelState} from "../ts/states/level";
import {TargetPartState} from "../ts/states/target_part";
import {StackData} from "./index";

export function TargetPart(props: {}): JSX.Element | null {
    const data = React.useContext(StackData);
    const target = data.get(TargetPartState.UI);
    if (!target) { return null; }
    const level = data.get(LevelState.Data)!;

    const point = level.board.hexCoords(target.hex);
    const style = {
        left: point.x,
        top: point.y,
    };
    let parts = target.targets.map(([part, valid]) =>
        <PartMenuItem
            key={part.id}
            part={part}
            valid={valid}
        ></PartMenuItem>);
    return <div className="partTargetMenu" style={style}>{parts}</div>;
}

export function PartMenuItem(props: {
    part: wasm.Part,
    valid: boolean,
}): JSX.Element {
    const data = React.useContext(StackData);
    const focus = data.get(Focus);
    const id: [wasm.Id<wasm.Creature>, wasm.Id<wasm.Part>] = [props.part.creatureId, props.part.id];
    return (
        <div
            className={props.valid?"playable":"unplayable"}
            onMouseDown={() => focus?.part.onClick?.(id)}
            onMouseEnter={() => focus?.part.onEnter?.(id)}
            onMouseLeave={() => focus?.part.onLeave?.(id)}
            >
            {props.part.name}
        </div>
    );
}