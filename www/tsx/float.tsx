import * as React from "react";

export function FloatText(props: {
    item: FloatText.Item,
}): JSX.Element {
    const style = Object.assign({
        left: props.item.pos.x,
        top: props.item.pos.y,
    }, props.item.style);
    return <div className="float" style={style}>{props.item.text}</div>;
}
export namespace FloatText {
    export interface Item {
        pos: DOMPointReadOnly,
        text: string,
        style?: React.CSSProperties,
    }
    export type ItemId = Item & {id: number};
    export class ItemSet {
        private _data: Set<Item> = new Set();
        private _ids: Map<Item, number> = new Map();
        private _nextId: number = 0;
        add(item: Item) {
            this._data.add(item);
            this._ids.set(item, this._nextId);
            this._nextId += 1;
        }
        delete(item: Item) {
            this._data.delete(item);
            this._ids.delete(item);
        }
        get all(): ItemId[] {
            return Array.from(this._data).map((f) => {
                let id: number = this._ids.get(f)!;
                return Object.assign({id}, f);
            })
        }
    }
}