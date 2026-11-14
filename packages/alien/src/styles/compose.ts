/** One independently removable CSS longhand at one exact condition. */
export type AtomicClass = Readonly<{
  property: string;
  /** Canonical media/supports/pseudo condition, or the empty string for the base rule. */
  condition: string;
  className: string;
}>;

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
  const selected: string[] = [];
  const seen = new Map<string, Set<string>>();
  const classes = new Set<string>();
  for (let index = styles.length - 1; index >= 0; index--) {
    const style = styles[index];
    if (!style) continue;
    for (let atomIndex = style.length - 1; atomIndex >= 0; atomIndex--) {
      const atom = style[atomIndex];
      if (!atom) continue;
      let properties = seen.get(atom.condition);
      if (!properties) {
        properties = new Set();
        seen.set(atom.condition, properties);
      }
      if (properties.has(atom.property)) continue;
      properties.add(atom.property);
      if (!classes.has(atom.className)) {
        selected.push(atom.className);
        classes.add(atom.className);
      }
    }
  }
  return selected.reverse().join(" ");
}
