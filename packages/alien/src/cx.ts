/** Joins class names, retaining each token once at its last occurrence.
 *
 * This does not resolve conflicts between different CSS classes: their
 * declarations still follow the stylesheet cascade.
 */
export function cx(...classes: readonly (string | false | null | undefined)[]): string {
  const seen = new Set<string>();
  const result: string[] = [];
  for (let index = classes.length - 1; index >= 0; index--) {
    const value = classes[index];
    if (typeof value !== "string") continue;
    const tokens = value.match(/[^\t\n\f\r ]+/g);
    if (!tokens) continue;
    for (let tokenIndex = tokens.length - 1; tokenIndex >= 0; tokenIndex--) {
      const token = tokens[tokenIndex];
      if (token !== undefined && !seen.has(token)) {
        seen.add(token);
        result.push(token);
      }
    }
  }
  return result.reverse().join(" ");
}
