import { copyFile, mkdir, mkdtemp, rename, rm } from "node:fs/promises";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { nativeFilename } from "../packages/alien-compiler/src/native-filename.ts";

const root = fileURLToPath(new URL("../", import.meta.url));
const target = join(root, "target");
const destination = join(root, "packages", "alien-compiler", "native");

const filenames: Partial<Record<NodeJS.Platform, string>> = {
  darwin: "libalien_napi.dylib",
  linux: "libalien_napi.so",
  win32: "alien_napi.dll",
};
const filename = filenames[process.platform];
if (!filename) throw new Error(`Unsupported native build platform: ${process.platform}`);

const binding = nativeFilename();
const environment = { ...process.env };
if (binding.endsWith("-musl.node")) {
  environment.RUSTFLAGS = [environment.RUSTFLAGS, "-C target-feature=-crt-static"]
    .filter(Boolean)
    .join(" ");
}

const build = Bun.spawnSync(
  [
    "cargo",
    "build",
    "--locked",
    "-p",
    "alien-napi",
    "--target-dir",
    target,
    ...(process.argv.includes("--release") ? ["--release"] : []),
  ],
  { cwd: root, env: environment, stdout: "inherit", stderr: "inherit" },
);
if (!build.success) process.exit(build.exitCode || 1);

await mkdir(destination, { recursive: true });
const temporaryDirectory = await mkdtemp(join(destination, ".alien-build-"));
try {
  const temporaryBinary = join(temporaryDirectory, "alien.node");
  await copyFile(
    join(target, process.argv.includes("--release") ? "release" : "debug", filename),
    temporaryBinary,
  );
  await rename(temporaryBinary, join(destination, binding));
} finally {
  await rm(temporaryDirectory, { recursive: true, force: true });
}
