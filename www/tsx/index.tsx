import * as React from "react";

export const index = (
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