import * as React from "react";

export function index(): [JSX.Element, React.RefObject<Index>] {
    let ref = React.createRef<Index>();
    return [<Index ref={ref}/>, ref];
}

// <params, state>
export class Index extends React.Component<{}, {}> {
    render() {
        return (
            <div className="center">
              <div id="leftSide" className="side"></div>
              <canvas id="mainCanvas" width="800" height="800"></canvas>
              <div className="side">
                <div id="baseRight" hidden>
                  <button id="endTurn">End Turn</button>
                </div>
              </div>
            </div>
        );
    }
}