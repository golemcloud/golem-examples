import { getEnvironment } from "wasi:cli/environment@0.2.0";

let env: Map<string, string> | undefined = undefined;

export const envVarKeys = {
  COMPONENT_ONE_ID: "COMPONENT_ONE_ID",
  COMPONENT_TWO_ID: "COMPONENT_TWO_ID",
  COMPONENT_THREE_ID: "COMPONENT_THREE_ID",
};

function getEnv(key: string): string | undefined {
  if (env === undefined) {
    env = new Map();
    for (const [key, value] of getEnvironment()) {
      env.set(key, value);
    }
  }

  return env.get(key);
}

function mustGetEnv(key: string): string {
  const value = getEnv(key);
  if (value == undefined) {
    throw new Error(`Expected environment variable is missing: ${key}`);
  }
  return value;
}

export interface Uri {
  value: string;
}

function getComponentWorkerURN(componentID: string, workerName: string): string {
  return `urn:worker:${componentID}/${workerName}`;
}

export function getComponentOneWorkerURN(workerName: string): Uri {
  return {
    value: getComponentWorkerURN(mustGetEnv(envVarKeys.COMPONENT_ONE_ID), workerName),
  };
}

export function getComponentTwoWorkerURN(workerName: string): Uri {
  return {
    value: getComponentWorkerURN(mustGetEnv(envVarKeys.COMPONENT_TWO_ID), workerName),
  };
}

export function getComponentThreeWorkerURN(workerName: string): Uri {
  return {
    value: getComponentWorkerURN(mustGetEnv(envVarKeys.COMPONENT_THREE_ID), workerName),
  };
}
