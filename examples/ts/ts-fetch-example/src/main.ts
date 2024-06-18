import {asyncToSyncAsResult} from "@golemcloud/golem-ts";
import {GolemTsFetchExampleApi} from "./interfaces/golem-ts-fetch-example-api";

let result: any

export const api: typeof GolemTsFetchExampleApi = {
    getLastResult(): string {
        return JSON.stringify(result);
    },
    fetchJson(url: string): string {
        result = asyncToSyncAsResult(fetch(url).then(response => response.json()));
        console.log(result);
        return JSON.stringify(result);
    },
}
