import type { ProjectTokenNames } from "../config";
export type TokenReference<Family extends string, Name extends string = string> = Readonly<{
  kind: `${Family}-token`;
  path: `${Family}.${Name}`;
}>;

export type TokenSet<Family extends string, Name extends string> = {
  readonly [Key in Name | ProjectTokenNames<Family>]: TokenReference<Family, Key>;
};

export function defineTokens<const Family extends string, const Name extends string>(
  family: Family,
  names: readonly Name[],
): TokenSet<Family, Name> {
  const builtin = Object.fromEntries(
    names.map((name) => [name, { kind: `${family}-token`, path: `${family}.${name}` }]),
  );
  return withProjectTokens(family, builtin) as TokenSet<Family, Name>;
}

/** Custom names are checked by generated types and the compiler, never resolved to CSS at runtime. */
export function withProjectTokens<Family extends string, Builtin extends object>(
  family: Family,
  builtin: Builtin,
): Builtin & Readonly<Record<ProjectTokenNames<Family>, TokenReference<Family>>> {
  return new Proxy(builtin, {
    get(target, key, receiver) {
      if (typeof key !== "string" || Reflect.has(target, key))
        return Reflect.get(target, key, receiver);
      return { kind: `${family}-token`, path: `${family}.${key}` };
    },
  }) as Builtin & Readonly<Record<ProjectTokenNames<Family>, TokenReference<Family>>>;
}
