import type { NativeCompiler } from "./native.ts";

const light = `
  *, *::before, *::after { box-sizing: border-box; }
  body { margin: 0; }
`;

const medium = `
  :where(img, video) { display: block; max-inline-size: 100%; block-size: auto; }
  :where(button, input, select, textarea) { font: inherit; }
`;

const strong = `
  :where(h1, h2, h3, h4, h5, h6, p, figure, blockquote, dl, dd, ul, ol) { margin: 0; }
  :where(h1, h2, h3, h4, h5, h6) { font: inherit; }
  :where(table) { border-collapse: collapse; border-spacing: 0; }
`;

export function resetCss(level: NativeCompiler["reset"]): string {
  switch (level) {
    case "light":
      return `@layer alien-reset {${light}}`;
    case "medium":
      return `@layer alien-reset {${light}${medium}}`;
    case "strong":
      return `@layer alien-reset {${light}${medium}${strong}}`;
    default:
      return "";
  }
}
