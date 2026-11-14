import { familySync, GLIBC, MUSL } from "detect-libc";

export function nativeFilename(): string {
  const { platform, arch } = process;
  if (!["darwin", "linux", "win32"].includes(platform) || !["x64", "arm64"].includes(arch)) {
    throw new Error(`Alien has no native compiler for ${platform}/${arch}.`);
  }
  if (platform !== "linux") return `alien-${platform}-${arch}.node`;

  const family = familySync();
  if (family !== GLIBC && family !== MUSL) {
    throw new Error("Alien could not detect the Linux libc (glibc or musl).");
  }
  return `alien-linux-${arch}-${family === MUSL ? "musl" : "gnu"}.node`;
}
