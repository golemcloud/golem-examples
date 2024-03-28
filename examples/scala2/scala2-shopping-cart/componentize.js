import { componentize } from "@bytecodealliance/componentize-js";
import { readFile, writeFile } from "node:fs/promises";
import { resolve } from "node:path";

const jsSource = await readFile("target/dist/main.js", "utf8");

const { component } = await componentize(jsSource, {
  witPath: resolve("wit"),
  enableStdout: true,
  preview2Adapter: "adapters/tier1/wasi_snapshot_preview1.wasm",
});

await writeFile("target/dist/component-name.wasm", component);
