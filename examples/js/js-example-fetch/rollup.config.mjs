import resolve from "@rollup/plugin-node-resolve";

export default {
    input: 'src/main.js',
    output: {
        file: 'out/main.js',
        format: 'esm'
    },
    external: ["golem:api/host@0.2.0"],
    plugins: [resolve()],
};