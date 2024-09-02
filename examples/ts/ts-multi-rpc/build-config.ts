import * as path from "node:path";
import { Dependencies } from "./src/build-tools/build-tools";

export const pckNs = "pack-ns";
export const outDir = "out";
export const componentsDir = path.join("src", "components");
export const libDir = path.join("src", "lib");
export const generatedDir = "generated";
export const componentTemplateDir = path.join("component-template", "component");

// Defines worker to worker RPC dependencies
export const componentDependencies: Dependencies = {
  "component-one": ["component-two", "component-three"],
  "component-two": ["component-three"],
};
