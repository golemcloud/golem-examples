import { componentize } from '@bytecodealliance/componentize-js';
import { readFile, writeFile, mkdir } from 'node:fs/promises';
import { resolve } from 'node:path';

const jsSource = await readFile('main.js', 'utf8');

const { component } = await componentize(jsSource, {
  witPath: resolve('wit'),
  enableStdout: true,
  preview2Adapter: 'adapters/tier2/wasi_snapshot_preview1.wasm'
});

await mkdir('out', { recursive: true });
await writeFile('out/component_name.wasm', component);
