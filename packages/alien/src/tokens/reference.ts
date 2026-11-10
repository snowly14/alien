export type TokenReference<Family extends string, Name extends string = string> = Readonly<{
  kind: `${Family}-token`;
  path: `${Family}.${Name}`;
}>;

export function defineTokens<const Family extends string, const Name extends string>(
  family: Family,
  names: readonly Name[],
) {
  return Object.fromEntries(
    names.map((name) => [name, { kind: `${family}-token`, path: `${family}.${name}` }]),
  ) as { readonly [Key in Name]: TokenReference<Family, Key> };
}
