import * as React from "react";

import * as wasm from "../wasm";

export function Entity(props: {
    entity: wasm.Entity,
}): JSX.Element {
    let status = Array.from(props.entity.status.values())
        .map(s => <span className="uibox" key={s.name}>{s.name}</span>);
    return <div>{status}</div>;
}