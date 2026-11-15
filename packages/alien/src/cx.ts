import { type AtomicClass, composeAtomicStyles } from "./styles/compose";

export type { AtomicClass } from "./styles/compose";
export type ClassInput = string | readonly (AtomicClass | string)[] | false | null | undefined;

/** Decode the compact representation produced by experimental atomic generation. */
function decodeAtomicToken(token: string): AtomicClass {
  const fields = token.split("~");
  const [className, property, condition = "", priority, group, start, count] = fields;
  if (!className || !property || ![2, 3, 4, 7].includes(fields.length))
    throw new Error("Invalid compiled atomic style token");
  if (priority !== undefined && priority !== "0" && priority !== "1")
    throw new Error("Invalid compiled atomic style priority");
  const atom: AtomicClass = { className, property, condition, important: priority === "1" };
  if (fields.length !== 7) return atom;
  const variantStart = Number(start);
  const variantCount = Number(count);
  if (
    (group !== "p" && group !== "m") ||
    !start ||
    !count ||
    !Number.isSafeInteger(variantStart) ||
    variantStart < 0 ||
    !Number.isSafeInteger(variantCount) ||
    variantCount < 1
  )
    throw new Error("Invalid compiled atomic style ordering");
  return { ...atom, orderGroup: group === "p" ? "padding" : "margin", variantStart, variantCount };
}

/**
 * Compose atomic descriptors with last-style-wins semantics. Ordinary class
 * strings remain opaque: their CSS follows the stylesheet cascade.
 * Shorthands in descriptors must already be expanded to longhand atoms.
 */
export function cx(...classes: readonly ClassInput[]): string {
  // Opaque classes get a distinct synthetic property per class token, so they
  // can be deduplicated without claiming to know their CSS declarations.
  // Conditions use a disjoint namespace from compiler-provided conditions.
  const styles = classes.map((value): readonly AtomicClass[] | false | null | undefined => {
    if (!value) return undefined;
    if (typeof value !== "string") {
      let entries: readonly (AtomicClass | string)[] = value;
      const packed = value.length === 1 ? value[0] : undefined;
      if (typeof packed === "string" && packed.startsWith("@")) {
        const [marker, ...tokens] = packed.split(" ");
        const prefix = marker?.slice(1) ?? "";
        if (
          !/^[A-Za-z_][A-Za-z0-9_-]*$/.test(prefix) ||
          !tokens.length ||
          tokens.some((token) => !token)
        )
          throw new Error("Invalid packed atomic style token");
        entries = tokens.map((token) => prefix + token);
      }
      return entries.map((entry) => {
        const atom = typeof entry === "string" ? decodeAtomicToken(entry) : entry;
        return { ...atom, condition: `atomic:${atom.condition}` };
      });
    }
    return (value.match(/[^\t\n\f\r ]+/g) ?? []).map((className) => ({
      className,
      property: className,
      condition: "opaque",
    }));
  });
  return composeAtomicStyles(styles);
}
