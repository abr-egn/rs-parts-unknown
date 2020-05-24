import * as React from "react";

import * as wasm from "../wasm";

export function Entity(props: {
    entity: wasm.Entity,
}): JSX.Element {
    let status = Array.from(props.entity.status.values())
        .map(s => <div className="status" key={s.name}>{s.name}</div>);
    return <div className="entity">{status}</div>;
}