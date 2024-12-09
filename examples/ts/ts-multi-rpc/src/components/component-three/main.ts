import { ComponentThreeApi } from "./binding/component-three";

let state = BigInt(0);

export const componentThreeApi: ComponentThreeApi = {
  add(value: bigint) {
    console.log(`Adding ${value} to the counter`);
    state += value;
  },
  get() {
    return state;
  },
};
