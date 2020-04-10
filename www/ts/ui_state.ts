export type UiStateKey<T> = {
    new (...args: any[]): T;
}

export class UiState {
    private _chunks: Map<UiStateKey<any>, any> = new Map();

    get<T>(key: UiStateKey<T>): T | undefined {
        return this._chunks.get(key);
    }

    build<T>(key: UiStateKey<T>, ...args: any[]): T {
        let chunk;
        if (chunk = this.get(key)) {
            return chunk;
        }
        chunk = new key(...args);
        this._chunks.set(key, chunk);
        return chunk;
    }
}