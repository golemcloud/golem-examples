import { ComponentOneApi } from "./generated/component-one";
import { ComponentTwoApi } from "pack-ns:component-two-stub/stub-component-two";
import { ComponentThreeApi } from "pack-ns:component-three-stub/stub-component-three";
import * as cfg from "../../lib/cfg";
import { getSelfMetadata } from "golem:api/host@1.1.0";

let state = BigInt(0);

export const componentOneApi: ComponentOneApi = {
  add(value: bigint) {
    console.log(`Adding ${value} to the counter`);

    const workerName = getSelfMetadata().workerId.workerName;

    const componentTwoWorkerURN = cfg.getComponentTwoWorkerURN(workerName);
    console.log(`Calling component two: ${componentTwoWorkerURN}`);
    const componentTwo = new ComponentTwoApi(componentTwoWorkerURN);
    componentTwo.blockingAdd(value);

    const componentThreeWorkerURN = cfg.getComponentThreeWorkerURN(workerName);
    console.log(`Calling component three: ${componentThreeWorkerURN}`);
    const componentThree = new ComponentThreeApi(componentThreeWorkerURN);
    componentThree.blockingAdd(value);

    state += value;
  },
  get() {
    return state;
  },
};
