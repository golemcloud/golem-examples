import * as path from "node:path";
import fs from "node:fs";

import {InputOptions, OutputOptions, rollup} from "rollup";

import rollupPluginNodeResolve from "@rollup/plugin-node-resolve";
import rollupPluginTypeScript, {RollupTypescriptOptions} from "@rollup/plugin-typescript";

import {cmd, cmdArg, Commands, fsMatch, getComponentNameFromArgs, main} from "./src/build-tools/build-tools";
import {run, runCapture} from "./src/lib/process";

import * as cfg from "./build-config";

const commands: Commands = {
    fmt: cmd(prettierWrite, "format using prettier"),
    lint: cmd(() => eslint(false), "lint project using eslint"),
    fix: cmd(fix, "format, lint and fix project using prettier and eslint"),
    generateNewComponent: cmdArg(
        generateNewComponents,
        "generates new component from template, expects <component-name>",
    ),
    rollupComponent: cmdArg(
        rollupComponentCmd,
        "Runs rollup for the specified component, expects <component-name>",
    ),
    deploy: cmd(deploy, "deploy (create or update) all components"),
    deployComponent: cmdArg(
        deployComponentCmd,
        "deploy (create or update) the specified component, expects <component-name>",
    ),
    test: cmd(test, "run tests"),
};

const componentNames: string[] = fs
    .readdirSync(cfg.componentsDir, {withFileTypes: true})
    .filter((entry) => entry.isDirectory())
    .map((entry) => entry.name);

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

async function rollupComponentCmd(args: string[]) {
    return rollupComponent(getComponentNameFromArgs(args));
}

async function rollupComponent(componentName: string) {
    const componentDir = path.join(cfg.componentsDir, componentName);
    const mainTs = path.join(cfg.componentsDir, componentName, "main.ts");
    const componentBuildDir = path.join(cfg.outDir, "build", componentName);
    const mainJs = path.join(componentBuildDir, "main.js");
    const generatedInterfacesDir = path.join(componentDir, cfg.generatedDir, "interfaces");

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
}

async function generateNewComponents(args: string[]) {
    const componentName = getComponentNameFromArgs(args);
    const componentDir = path.join(cfg.componentsDir, componentName);

    if (fs.existsSync(componentDir)) {
        throw new Error(`${componentDir} already exists!`);
    }

    console.log(`Creating directory ${componentDir}`);
    fs.mkdirSync(componentDir, {recursive: true});

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
            template = template.replaceAll("compName", componentNameCamel);
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
    const linkedComponentsDir = path.join(cfg.outDir, "linked-components");
    const wasm = path.join(linkedComponentsDir, componentName + "-linked.wasm");
    return run("golem-cli", ["component", "add", "--non-interactive", "--component-name", componentName, wasm]);
}

async function test() {
    return run("npx", ["tsx", ...fsMatch({includePaths: ["test"], picoPattern: "test/**.test.ts"})]);
}

await main(commands);
