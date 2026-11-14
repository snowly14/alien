import { createHash } from "node:crypto";
import { readFile } from "node:fs/promises";
import { basename, relative, resolve } from "node:path";

export async function projectNamespace(root: string, explicit?: string): Promise<string> {
  if (explicit) return explicit;
  try {
    const manifest: unknown = JSON.parse(await readFile(resolve(root, "package.json"), "utf8"));
    if (
      typeof manifest === "object" &&
      manifest !== null &&
      "name" in manifest &&
      typeof manifest.name === "string" &&
      manifest.name
    )
      return manifest.name;
  } catch (error) {
    if (!(error instanceof Error && "code" in error && error.code === "ENOENT")) throw error;
  }
  return basename(root);
}

/** Stable across client/server builds and independent of the absolute checkout path. */
export function moduleIdentity(root: string, namespace: string, path: string) {
  const id = `${namespace}/${relative(root, path).replaceAll("\\", "/")}`;
  const digest = createHash("sha256").update(id).digest("hex");
  return { id, digest, prefix: digest.slice(0, 32) };
}
