import fs, { Dirent } from "node:fs";
import path from "node:path";
import picomatch from "picomatch";

export class Command {
  private constructor(
    description: string,
    run: (() => Promise<void>) | undefined,
    runArgs: ((args: string[]) => Promise<void>) | undefined,
  ) {
    this.description = description;
    this.run = run;
    this.runArgs = runArgs;
  }

  static cmd(run: () => Promise<void>, description: string): Command {
    return new Command(description, run, undefined);
  }

  static cmdArgs(runArgs: (args: string[]) => Promise<void>, description: string): Command {
    return new Command(description, undefined, runArgs);
  }

  readonly description: string;
  readonly run?: () => Promise<void>;
  readonly runArgs?: (args: string[]) => Promise<void>;
}

export const cmd = Command.cmd;
export const cmdArg = Command.cmdArgs;

export type Commands = { [key: string]: Command };

export function getComponentNameFromArgs(args: string[]) {
  if (args.length != 1) {
    throw new Error(`generateNewComponents expected exactly one argument (component-name), got: [${args.join(", ")}]`);
  }

  const componentName = args[0];
  if (componentName === undefined) {
    throw new Error("Undefined component name");
  }

  return componentName;
}

export async function main(commands: Commands) {
  const args = process.argv.splice(2);

  if (args.length == 0) {
    const maxCmdLen = Math.max(...Object.keys(commands).map((cmd) => cmd.length)) + 1;
    console.log("Available commands:");
    for (const [name, cmd] of Object.entries(commands)) {
      console.log(`  ${(name + ":").padEnd(maxCmdLen)} ${cmd.description}`);
    }
    return;
  }

  for (const cmd of args) {
    const command = commands[cmd];
    if (command === undefined) {
      throw new Error(`Command not found: ${cmd}`);
    }
    if (command.run !== undefined) {
      await command.run();
    } else if (command.runArgs != undefined) {
      await command.runArgs(args.splice(1));
      return;
    } else {
      throw new Error(
        `Illegal state: both command.run and command.runArgs is undefined, description: ${command.description}`,
      );
    }
  }
}

interface FsMatchOptions {
  includePaths?: string[];
  picoPattern?: string;
  direntFilter?: (dirent: Dirent) => boolean;
}

export function fsMatch(options: FsMatchOptions): string[] {
  const includePaths = options.includePaths ?? ["."];
  return includePaths.flatMap((includePath) => {
    if (!fs.existsSync(includePath)) return [];
    const picoMatcher = options.picoPattern !== undefined ? picomatch(options.picoPattern) : undefined;
    return fs
      .readdirSync(includePath, { withFileTypes: true, recursive: true })
      .map((entry): [Dirent, string] => [entry, path.join(entry.parentPath, entry.name)])
      .filter(([dirent, path]) => {
        if (options.direntFilter !== undefined && !options.direntFilter(dirent)) return false;
        if (picoMatcher !== undefined && !picoMatcher(path)) return false;
        return true;
      })
      .map(([, path]) => path);
  });
}
