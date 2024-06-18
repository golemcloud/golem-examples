import typescript from "@rollup/plugin-typescript";
import resolve from "@rollup/plugin-node-resolve";

export default {
    input: 'src/main.ts',
    output: {
        file: 'out/main.js',
        format: 'esm'
    },
    external: ["golem:api/host@0.2.0"],
    plugins: [resolve(), typescript()],
};