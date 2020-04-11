import {immerable} from "immer";

type Constructor = new (...args: any[]) => any;

export class UiData {
    [immerable] = true;

    private _chunks: Map<any, any> = new Map();

    get<C extends Constructor>(key: C): Readonly<InstanceType<C>> | undefined {
        return this._chunks.get(key);
    }

    build<C extends Constructor>(key: C, ...args: ConstructorParameters<C>): InstanceType<C> {
        let chunk;
        if (chunk = this._chunks.get(key)) {
            return chunk;
        }
        chunk = new key(...args);
        this._chunks.set(key, chunk);
        return chunk;
    }

    set<C extends Constructor>(key: C, ...args: ConstructorParameters<C>) {
        this._chunks.set(key, new key(...args));
    }
}