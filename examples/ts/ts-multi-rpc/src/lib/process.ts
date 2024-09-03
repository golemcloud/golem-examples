import child_process from "node:child_process";

export function run(command: string, args: string[]): Promise<void> {
  return new Promise((resolve, reject) => {
    const child = child_process.spawn(command, args);

    child.stdout.on("data", (data) => process.stdout.write(data));
    child.stderr.on("data", (data) => process.stderr.write(data));

    child.on("close", (code) => {
      if (code === 0) {
        resolve();
      } else {
        reject(new Error(`Command [${command} ${args.join(" ")}] failed with exit code ${code}`));
      }
    });

    child.on("error", (error) => reject(error));
  });
}

export interface RunResult {
  code: number | null;
  stdout: string;
  stderr: string;
  cmd: string;
}

export function runCapture(command: string, args: string[]): Promise<RunResult> {
  return new Promise((resolve, reject) => {
    const child = child_process.spawn(command, args);

    let stderr: string = "";
    let stdout: string = "";

    child.stdout.on("data", (data) => (stdout += data));
    child.stderr.on("data", (data) => (stderr += data));

    child.on("close", (code) => {
      resolve({
        code: code,
        stdout: stdout,
        stderr: stderr,
        cmd: `${command} ${args.join(" ")}`,
      });
    });

    child.on("error", (error) => reject(error));
  });
}
