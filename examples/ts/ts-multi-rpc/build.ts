import * as path from "node:path";
import fs from "node:fs";

import { InputOptions, OutputOptions, rollup } from "rollup";

import rollupPluginNodeResolve from "@rollup/plugin-node-resolve";
import rollupPluginTypeScript, { RollupTypescriptOptions } from "@rollup/plugin-typescript";

import { allDepsSorted, cmd, cmdArg, Commands, fsMatch, main, runTask } from "./src/build-tools/build-tools";
import { run, runCapture } from "./src/lib/process";

import * as cfg from "./build-config";

const commands: Commands = {
  fmt: cmd(prettierWrite, "format using prettier"),
  lint: cmd(() => eslint(false), "lint project using eslint"),
  fix: cmd(fix, "format, lint and fix project using prettier and eslint"),
  build: cmd(build, "build all components"),
  updateRpcStubs: cmd(updateRpcStubs, "update stubs based on componentDependencies"),
  generateNewComponent: cmdArg(
    generateNewComponents,
    "generates new component from template, expects <component-name>",
  ),
  deploy: cmd(deploy, "deploy (create or update) all components"),
  deployComponent: cmdArg(
    deployComponentCmd,
    "deploy (create or update) the specified component, expects <component-name>",
  ),
  test: cmd(test, "run tests"),
  clean: cmd(clean, "clean outputs and generated code"),
};

const componentNames: string[] = fs
  .readdirSync(cfg.componentsDir, { withFileTypes: true })
  .filter((entry) => entry.isDirectory())
  .map((entry) => entry.name);

async function build() {
  for (const componentName of componentNames) {
    await buildComponent(componentName);
  }
}

async function buildComponent(componentName: string) {
  console.log(`Build component: ${componentName}`);

  await generateBinding(componentName);
  await rollupComponent(componentName);
  await componentize(componentName);
  await stubCompose(componentName);
}

async function generateBinding(componentName: string) {
  const componentDir = path.join(cfg.componentsDir, componentName);
  const witDir = path.join(componentDir, "wit");
  const bindingDir = path.join(componentDir, cfg.generatedDir);

  return runTask({
    runMessage: `Generating bindings from ${witDir} into ${bindingDir}`,
    skipMessage: "binding generation",
    targets: [bindingDir],
    sources: [witDir],
    run: async () => {
      return run("npx", ["jco", "stubgen", witDir, "-o", bindingDir]);
    },
  });
}

async function prettierWrite() {
  return run("npx", ["prettier", ".", "--write"]);
}

async function eslint(fix: boolean) {
  const args = ["eslint", "--color"];
  if (fix) args.push("--fix");
  return run("npx", args);
}

async function fix() {
  await prettierWrite();
  await eslint(true);
}

async function rollupComponent(componentName: string) {
  const componentDir = path.join(cfg.componentsDir, componentName);
  const mainTs = path.join(cfg.componentsDir, componentName, "main.ts");
  const componentBuildDir = path.join(cfg.outDir, "build", componentName);
  const mainJs = path.join(componentBuildDir, "main.js");
  const generatedInterfacesDir = path.join(componentDir, cfg.generatedDir, "interfaces");

  return runTask({
    runMessage: `Rollup component: ${componentName}`,
    skipMessage: "component rollup",
    targets: [mainJs],
    sources: [componentDir, cfg.libDir, "build.ts", "package.json", "tsconfig.json"],
    run: async () => {
      const moduleRegex = /declare\s+module\s+"([^"]+)"/g;
      const externalInterfaces: string[] = fsMatch({
        includePaths: [generatedInterfacesDir],
        picoPattern: "**/*.d.ts",
      }).flatMap((path) =>
        [...fs.readFileSync(path).toString().matchAll(moduleRegex)].map((match) => {
          const moduleName = match[1];
          if (moduleName === undefined) {
            throw new Error(`Missing match for module name`);
          }
          return moduleName;
        }),
      );

      const tsOptions: RollupTypescriptOptions = {
        include: ["src/lib/**/*.ts", componentDir + "/**/*.ts"],
      };

      const input: InputOptions = {
        input: mainTs,
        external: externalInterfaces,
        plugins: [rollupPluginNodeResolve(), rollupPluginTypeScript(tsOptions)],
      };

      const output: OutputOptions = {
        file: mainJs,
        format: "esm",
      };

      const bundle = await rollup(input);
      await bundle.write(output);
      await bundle.close();
    },
  });
}

async function componentize(componentName: string) {
  const componentDir = path.join(cfg.componentsDir, componentName);
  const witDir = path.join(componentDir, "wit");
  const componentBuildDir = path.join(cfg.outDir, "build", componentName);
  const mainJs = path.join(componentBuildDir, "main.js");
  const componentWasm = path.join(componentBuildDir, "component.wasm");

  return runTask({
    runMessage: `Componentizing component: ${componentName}`,
    skipMessage: "componentize",
    targets: [componentWasm],
    sources: [mainJs],
    run: async () => {
      await run("npx", ["jco", "componentize", "-w", witDir, "-o", componentWasm, mainJs]);
    },
  });
}

async function stubCompose(componentName: string) {
  const componentBuildDir = path.join(cfg.outDir, "build", componentName);
  const componentWasm = path.join(componentBuildDir, "component.wasm");
  const componentsBuildDir = path.join(cfg.outDir, "components");
  const targetWasm = path.join(cfg.outDir, "components", componentName + ".wasm");

  const stubWasms: string[] = [];
  const deps = cfg.componentDependencies[componentName];
  if (deps !== undefined) {
    for (const componentName of deps) {
      stubWasms.push(path.join(cfg.outDir, "stub", componentName, "stub.wasm"));
    }
  }

  return runTask({
    runMessage: `Composing stubs [${stubWasms.join(", ")}] into component: ${componentName}`,
    skipMessage: "stub compose",
    targets: [targetWasm],
    sources: [componentWasm, ...stubWasms],
    run: async () => {
      let composeWasm = componentWasm;
      if (stubWasms.length > 0) {
        let srcWasm = componentWasm;
        let i = 0;
        for (const stubWasm of stubWasms) {
          i++;
          const prevComposeWasm = composeWasm;
          composeWasm = path.join(componentBuildDir, `compose-${i}-${path.basename(path.dirname(stubWasm))}.wasm`);
          const result = await runCapture("golem-cli", [
            "stubgen",
            "compose",
            "--source-wasm",
            srcWasm,
            "--stub-wasm",
            stubWasm,
            "--dest-wasm",
            composeWasm,
          ]);
          if (result.code !== 0) {
            if (result.stderr.includes("Error: no dependencies of component") && result.stderr.includes("were found")) {
              console.log(`Skipping composing ${stubWasm}, not used`);
              composeWasm = prevComposeWasm;
              continue;
            }

            if (result.stdout) {
              process.stderr.write(result.stdout);
            }
            if (result.stderr) {
              process.stderr.write(result.stderr);
            }

            throw new Error(`Command [${result.cmd}] failed with code: ${result.code}`);
          }
          srcWasm = composeWasm;
        }
      }

      fs.mkdirSync(componentsBuildDir, { recursive: true });
      fs.copyFileSync(composeWasm, targetWasm);
    },
  });
}

async function updateRpcStubs() {
  const stubs = allDepsSorted(cfg.componentDependencies);
  for (const stub of stubs) {
    await buildStubComponent(stub);
  }

  for (const [comp, deps] of Object.entries(cfg.componentDependencies)) {
    for (const dep of deps) {
      await addStubDependency(comp, dep);
    }
  }
}

async function buildStubComponent(componentName: string) {
  const componentDir = path.join(cfg.componentsDir, componentName);
  const srcWitDir = path.join(componentDir, "wit");
  const stubTargetDir = path.join(cfg.outDir, "stub", componentName);
  const destWasm = path.join(stubTargetDir, "stub.wasm");
  const destWitDir = path.join(stubTargetDir, "wit");

  return runTask({
    runMessage: `Building stub component for: ${componentName}`,
    skipMessage: "stub component build",
    targets: [destWasm, destWitDir],
    sources: [srcWitDir],
    run: async () => {
      return run("golem-cli", [
        "stubgen",
        "build",
        "--source-wit-root",
        srcWitDir,
        "--dest-wasm",
        destWasm,
        "--dest-wit-root",
        destWitDir,
      ]);
    },
  });
}

async function addStubDependency(componentName: string, depComponentName: string) {
  const stubTargetDir = path.join(cfg.outDir, "stub", depComponentName);
  const srcWitDir = path.join(stubTargetDir, "wit");
  const dstComponentDir = path.join(cfg.componentsDir, componentName);
  const dstWitDir = path.join(dstComponentDir, "wit");
  const dstWitDepDir = path.join(dstComponentDir, dstWitDir, "deps", `${cfg.pckNs}_${componentName}`);
  const dstWitDepStubDir = path.join(dstComponentDir, dstWitDir, "deps", `${cfg.pckNs}_${componentName}-stub`);

  return runTask({
    runMessage: `Adding stub dependency for ${depComponentName} to ${componentName}`,
    skipMessage: "add stub dependency",
    targets: [dstWitDepDir, dstWitDepStubDir],
    sources: [srcWitDir],
    run: async () => {
      return run("golem-cli", [
        "stubgen",
        "add-stub-dependency",
        "--overwrite",
        "--stub-wit-root",
        srcWitDir,
        "--dest-wit-root",
        dstWitDir,
      ]);
    },
  });
}

async function generateNewComponents(args: string[]) {
  const componentName = getComponentNameFromArgs(args);
  const componentDir = path.join(cfg.componentsDir, componentName);

  if (fs.existsSync(componentDir)) {
    throw new Error(`${componentDir} already exists!`);
  }

  console.log(`Creating directory ${componentDir}`);
  fs.mkdirSync(componentDir, { recursive: true });

  const entries = fs.readdirSync(cfg.componentTemplateDir, {
    recursive: true,
    withFileTypes: true,
  });

  for (const entry of entries) {
    const relEntryPath = path.relative(cfg.componentTemplateDir, entry.parentPath);
    if (entry.isDirectory()) {
      const targetPath = path.join(componentDir, relEntryPath, entry.name);
      console.log(`Creating directory ${targetPath}`);
      fs.mkdirSync(targetPath);

      continue;
    }

    if (entry.name.endsWith(".template")) {
      const sourcePath = path.join(entry.parentPath, entry.name);
      const targetPath = path.join(componentDir, relEntryPath, entry.name.replaceAll(".template", ""));
      console.log(`Generating ${targetPath} from ${sourcePath}`);

      const dashToPascal = (str: string): string =>
        str
          .split("-")
          .map((s) => s.substring(0, 1).toUpperCase() + s.substring(1))
          .join("");
      const componentNamePascal = dashToPascal(componentName);
      const componentNameCamel = componentNamePascal.substring(0, 1).toLowerCase() + componentNamePascal.substring(1);

      let template = fs.readFileSync(sourcePath).toString();
      template = template.replaceAll("pck-ns", cfg.pckNs);
      template = template.replaceAll("comp-name", componentName);
      template = template.replaceAll("componentName", componentNameCamel);
      template = template.replaceAll("CompName", componentNamePascal);

      fs.writeFileSync(targetPath, new Uint8Array(Buffer.from(template)));

      continue;
    }

    const sourcePath = path.join(entry.parentPath, entry.name);
    const targetPath = path.join(componentDir, relEntryPath, entry.name);
    console.log(`Copying ${sourcePath} to ${targetPath}`);
    fs.copyFileSync(sourcePath, targetPath);
  }
}

async function deploy() {
  for (const componentName of componentNames) {
    await deployComponent(componentName);
  }
}

async function deployComponentCmd(args: string[]) {
  return deployComponent(getComponentNameFromArgs(args));
}

async function deployComponent(componentName: string) {
  console.log(`Deploying ${componentName}`);
  const componentsTargetDir = path.join(cfg.outDir, "components");
  const wasm = path.join(componentsTargetDir, componentName + ".wasm");
  return run("golem-cli", ["component", "add", "--non-interactive", "--component-name", componentName, wasm]);
}

async function test() {
  return run("npx", ["tsx", ...fsMatch({ includePaths: ["test"], picoPattern: "test/**.test.ts" })]);
}

async function clean() {
  const paths = ["out"];
  for (const componentName of componentNames) {
    paths.push(path.join(cfg.componentsDir, componentName, cfg.generatedDir));
  }

  for (const path of paths) {
    console.log(`Deleting ${path}`);
    fs.rmSync(path, { recursive: true, force: true });
  }
}

function getComponentNameFromArgs(args: string[]) {
  if (args.length != 1) {
    throw new Error(`generateNewComponents expected exactly one argument (component-name), got: [${args.join(", ")}]`);
  }

  const componentName = args[0];
  if (componentName === undefined) {
    throw new Error("Undefined component name");
  }

  return componentName;
}

await main(commands);
