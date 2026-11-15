/** One independently removable CSS longhand at one exact condition. */
export type AtomicClass = Readonly<{
  property: string;
  /** Canonical media/supports/pseudo condition, or the empty string for the base rule. */
  condition: string;
  className: string;
  /** Whether the declaration has CSS !important priority. */
  important?: boolean;
  /** Compiler-provided classes in increasing cascade order. */
  variants?: readonly string[];
  /** Compact range of compiler-generated ordered classes. */
  variantStart?: number;
  variantCount?: number;
  /** Properties that require a shared ordering, such as logical padding. */
  orderGroup?: string;
}>;

// Some CSS properties overlap without one being a simple duplicate of the
// other. In particular, logical sides depend on writing mode and direction;
// removing either class would silently discard part of the earlier style.
const overlaps: Readonly<Record<string, readonly string[]>> = {
  padding: [
    "padding-top",
    "padding-right",
    "padding-bottom",
    "padding-left",
    "padding-inline",
    "padding-block",
  ],
  margin: [
    "margin-top",
    "margin-right",
    "margin-bottom",
    "margin-left",
    "margin-inline",
    "margin-block",
  ],
  "padding-inline": ["padding-top", "padding-right", "padding-bottom", "padding-left"],
  "padding-block": ["padding-top", "padding-right", "padding-bottom", "padding-left"],
  "margin-inline": ["margin-top", "margin-right", "margin-bottom", "margin-left"],
  "margin-block": ["margin-top", "margin-right", "margin-bottom", "margin-left"],
  flex: ["flex-grow", "flex-shrink", "flex-basis"],
  gap: ["row-gap", "column-gap"],
  inset: ["top", "right", "bottom", "left"],
  overflow: ["overflow-x", "overflow-y"],
  "border-radius": [
    "border-top-left-radius",
    "border-top-right-radius",
    "border-bottom-right-radius",
    "border-bottom-left-radius",
  ],
  "border-width": [
    "border-top-width",
    "border-right-width",
    "border-bottom-width",
    "border-left-width",
  ],
  "border-style": [
    "border-top-style",
    "border-right-style",
    "border-bottom-style",
    "border-left-style",
  ],
  "border-color": [
    "border-top-color",
    "border-right-color",
    "border-bottom-color",
    "border-left-color",
  ],
};

function overlapsPartially(left: string, right: string): boolean {
  return !!overlaps[left]?.includes(right) || !!overlaps[right]?.includes(left);
}

function hasOrderedVariants(atom: AtomicClass): boolean {
  return (
    !!atom.variants ||
    (Number.isSafeInteger(atom.variantStart) &&
      Number.isSafeInteger(atom.variantCount) &&
      (atom.variantStart ?? -1) >= 0 &&
      (atom.variantCount ?? 0) > 0)
  );
}

function compactIndex(index: number): string {
  const alphabet = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
  let result = "";
  for (;;) {
    result = alphabet[index % 52] + result;
    if (index < 52) return result;
    index = Math.floor(index / 52) - 1;
  }
}

function orderedClass(atom: AtomicClass, rank: number): string | undefined {
  if (atom.variants) return atom.variants[rank];
  const start = atom.variantStart;
  const count = atom.variantCount;
  if (start === undefined || count === undefined || rank >= count) return undefined;
  const suffix = compactIndex(start);
  if (!atom.className.endsWith(suffix)) return undefined;
  return atom.className.slice(0, -suffix.length) + compactIndex(start + rank);
}

/**
 * Resolves compiler-expanded atoms before class names are joined.
 *
 * Shorthands must be expanded into independently emitted longhands by the
 * compiler. A grouped class cannot be used here: removing it would discard
 * unrelated declarations. Different conditions retain ordinary CSS cascading.
 */
export function composeAtomicStyles(
  styles: readonly (readonly AtomicClass[] | false | null | undefined)[],
): string {
  const selected: AtomicClass[] = [];
  const seen = new Map<string, Set<string>>();
  const important = new Map<string, Set<string>>();
  const propertiesByCondition = new Map<string, Map<string, AtomicClass>>();
  for (const style of styles) {
    if (!style) continue;
    for (const atom of style) {
      if (atom.condition !== "opaque") {
        let properties = propertiesByCondition.get(atom.condition);
        if (!properties) {
          properties = new Map();
          propertiesByCondition.set(atom.condition, properties);
        }
        for (const [property, previous] of properties) {
          if (
            overlapsPartially(property, atom.property) &&
            !(
              atom.orderGroup &&
              atom.orderGroup === previous.orderGroup &&
              hasOrderedVariants(atom) &&
              hasOrderedVariants(previous)
            )
          ) {
            throw new Error(
              `Cannot compose overlapping atomic CSS properties ${property} and ${atom.property}`,
            );
          }
        }
        properties.set(atom.property, atom);
      }
      if (!atom.important) continue;
      let properties = important.get(atom.condition);
      if (!properties) {
        properties = new Set();
        important.set(atom.condition, properties);
      }
      properties.add(atom.property);
    }
  }
  const classes = new Set<string>();
  for (let index = styles.length - 1; index >= 0; index--) {
    const style = styles[index];
    if (!style) continue;
    for (let atomIndex = style.length - 1; atomIndex >= 0; atomIndex--) {
      const atom = style[atomIndex];
      if (!atom) continue;
      if (!atom.important && important.get(atom.condition)?.has(atom.property)) continue;
      let properties = seen.get(atom.condition);
      if (!properties) {
        properties = new Set();
        seen.set(atom.condition, properties);
      }
      if (properties.has(atom.property)) continue;
      properties.add(atom.property);
      if (!classes.has(atom.className)) {
        selected.push(atom);
        classes.add(atom.className);
      }
    }
  }
  const ranks = new Map<string, Map<string, number>>();
  return selected
    .reverse()
    .map((atom) => {
      if (!atom.orderGroup || !hasOrderedVariants(atom)) return atom.className;
      let condition = ranks.get(atom.condition);
      if (!condition) {
        condition = new Map();
        ranks.set(atom.condition, condition);
      }
      const rank = condition.get(atom.orderGroup) ?? 0;
      condition.set(atom.orderGroup, rank + 1);
      const selectedClass = orderedClass(atom, rank);
      if (!selectedClass)
        throw new Error(`Too many ordered atomic properties in ${atom.orderGroup}`);
      return selectedClass;
    })
    .join(" ");
}
