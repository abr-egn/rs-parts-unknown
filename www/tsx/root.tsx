import * as React from "react";
import * as ReactDOM from "react-dom";

const ROOT = document.getElementById('root')!;

export function RootPortal(props: {children: React.ReactNode}): JSX.Element {
    return ReactDOM.createPortal(props.children, ROOT);
}