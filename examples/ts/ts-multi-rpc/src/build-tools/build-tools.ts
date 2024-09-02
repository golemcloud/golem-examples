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

export interface Task {
  runMessage: string;
  skipMessage: string;
  targets: string[];
  sources: string[];
  run: () => Promise<void>;
}

export async function runTask(task: Task) {
  let run = task.targets.length == 0;

  upToDateCheck: for (const target of task.targets) {
    let targetInfo;
    try {
      targetInfo = fs.statSync(target);
    } catch (error) {
      if (error instanceof Error && "code" in error && error.code == "ENOENT") {
        run = true;
        break;
      }
      throw error;
    }

    let targetModifiedMs = Number.MAX_SAFE_INTEGER;

    if (targetInfo.isDirectory()) {
      const targets = fs.readdirSync(target, {
        recursive: true,
        withFileTypes: true,
      });
      for (const target of targets) {
        if (target.isDirectory()) continue;
        const targetInfo = fs.statSync(path.join(target.parentPath, target.name));
        if (targetModifiedMs > targetInfo.mtimeMs) {
          targetModifiedMs = targetInfo.mtimeMs;
        }
      }
    } else {
      targetModifiedMs = targetInfo.mtimeMs;
    }

    for (const source of task.sources) {
      const sourceInfo = fs.statSync(source);

      if (!sourceInfo.isDirectory()) {
        if (sourceInfo.mtimeMs > targetModifiedMs) {
          run = true;
          break upToDateCheck;
        }
        continue;
      }

      const sources = fs.readdirSync(source, {
        recursive: true,
        withFileTypes: true,
      });
      for (const source of sources) {
        if (source.isDirectory()) continue;
        const sourceInfo = fs.statSync(path.join(source.parentPath, source.name));
        if (sourceInfo.mtimeMs > targetModifiedMs) {
          run = true;
          break upToDateCheck;
        }
      }
    }
  }

  if (!run) {
    console.log(`${task.targets.join(",")} is up to date, skipping ${task.skipMessage}`);
    return;
  }

  console.log(task.runMessage);
  await task.run();
}

export type Dependencies = { [key: string]: string[] };

export function allDepsSorted(dependencies: Dependencies): string[] {
  const allDepsSet = new Set<string>();
  for (const deps of Object.values(dependencies).values()) {
    for (const dep of deps) {
      allDepsSet.add(dep);
    }
  }
  return Array.from(allDepsSet).sort();
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
