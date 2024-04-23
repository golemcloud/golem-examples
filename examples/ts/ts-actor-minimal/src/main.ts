import { PackNameApi } from './interfaces/pack-name-api.js';

let state = BigInt(0);

export const api: typeof PackNameApi = {
    add(value: bigint) {
        console.log(`Adding ${value} to the counter`);
        state += value;
    },
    get() {
        return state;
    }
}
