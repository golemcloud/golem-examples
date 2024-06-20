import {asyncToSyncAsResult} from "@golemcloud/golem-ts";
import { PackNameApi } from './interfaces/pack-name-api';

let result: any

export const api: typeof PackNameApi = {
    getLastResult(): string {
        return JSON.stringify(result);
    },
    fetchJson(url: string): string {
        result = asyncToSyncAsResult(fetch(url).then(response => response.json()));
        console.log(result);
        return JSON.stringify(result);
    },
}
