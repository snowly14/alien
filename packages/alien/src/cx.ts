import { type AtomicClass, composeAtomicStyles } from "./styles/compose";

export type { AtomicClass } from "./styles/compose";
export type ClassInput = string | readonly AtomicClass[] | false | null | undefined;

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
    if (typeof value !== "string")
      return value.map((atom) => ({ ...atom, condition: `atomic:${atom.condition}` }));
    return (value.match(/[^\t\n\f\r ]+/g) ?? []).map((className) => ({
      className,
      property: className,
      condition: "opaque",
    }));
  });
  return composeAtomicStyles(styles);
}
