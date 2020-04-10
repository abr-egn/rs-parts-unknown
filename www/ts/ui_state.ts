import {immerable} from "immer";

type BaseMap<K, V> = Map<K, V>;
const BaseMap = Map;

export namespace UiState {

    type Constructor = new (...args: any[]) => any;

    export class Map {
        [immerable] = true;

        private _chunks: BaseMap<any, any> = new BaseMap();

        get<C extends Constructor>(key: C): InstanceType<C> | undefined {
            return this._chunks.get(key);
        }

        build<C extends Constructor>(key: C, ...args: ConstructorParameters<C>): InstanceType<C> {
            let chunk;
            if (chunk = this.get(key)) {
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

}