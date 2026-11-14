# Supported Alien syntax

This document describes implemented behavior. The broader [initial proposal](./docs/proposals/initial-api.md) is historical.

## Policy

`alien.globals({ mode: "free" | "strict", ... })` selects one project policy. Without a file, the policy is `free`.

| Values | Free | Strict |
| --- | --- | --- |
| Colors | Tokens, color helpers, supported CSS color strings, legacy `custom(color)` | Color tokens and token-derived `alpha` / `mix`; `transparent` and `currentColor` |
| Padding, margin, gaps | Space tokens, compatible dimensions, legacy `custom(dimension)` | Space tokens; applicable keywords such as `auto` / `normal` |
| Radius | Radius tokens or compatible dimensions | Radius tokens |
| Font weight | Weight tokens, numbers, CSS keywords, legacy `custom(number)` | Weight tokens and CSS keywords |
| Font size | Size tokens or compatible dimensions | Size tokens |
| Line height | Tokens, numbers, dimensions or `normal` | Line-height tokens or `normal` |
| Transition/animation time | Duration tokens or `ms()` | Duration tokens |
| Layout dimensions and border widths | Compatible dimensions | Compatible dimensions |
| Other CSS keywords | Typed literal strings or `css.*` | Same |

`css.display.flex` and `"flex"` mean the same thing. Keyword compatibility follows their value, not which `css.*` property produced it. Font-weight scale labels such as `medium` and `semibold` belong to `tokens.fontWeight`, not `css.fontWeight`.

## Project tokens

```ts
import { alien } from "alien";
import { hex, ms, rem } from "alien/values";

export default alien.globals({
  mode: "strict",
  color: { brand: hex("#168043") },
  space: { control: rem(0.625) },
  radius: { card: "0.75rem" },
  fontWeight: { label: 650 },
  fontSize: { label: "0.875rem" },
  lineHeight: { label: 1.4 },
  duration: { reveal: ms(240) },
});
```

Supported custom families are `color`, `space`, `radius`, `fontWeight`, `fontSize`, `lineHeight`, and `duration`. Names are ASCII identifiers; reserved prototype-related names are rejected. Existing built-in names cannot be replaced. Duplicate declarations and invalid values fail before any component is compiled. Numeric/dimensional tokens use literal values; token aliases in those families are not supported. Color expressions can reference built-in colors. Custom palettes, themes, overrides, custom breakpoints, custom shadows and custom easing curves are not implemented.

Configuration permits imports and exactly one direct `alien.globals({...})` call, optionally default-exported. It cannot execute arbitrary statements or read environment variables. Run `alien prepare` before TypeScript checking and include `alien-env.d.ts` in that application's TypeScript project. The generated file is ignored by Git. Deleted names are removed on regeneration.

## Optional CSS reset

Configure the reset independently of the project mode.

```ts
import { alien } from "alien";

export default alien.globals({ reset: "medium" });
```

- `false` or omitted disables the reset.
- `"light"` applies border-box sizing and removes the default body margin.
- `"medium"` adds responsive image and video defaults and inherits fonts on form controls.
- `"strong"` also removes default margins from common text, list and figure elements, inherits heading typography, and collapses table borders.

The Vite plugin applies the reset to HTML entries. The `alien-reset` layer lets normal unlayered application styles override it. List markers and keyboard focus outlines remain available.

## Static component syntax

```ts
import { alien, css, tokens } from "alien";
import { alpha, percent, px } from "alien/values";

export const styles = alien.create({
  button: {
    display: css.display.flex,
    padding: tokens.space.control,
    color: tokens.color.brand,
    border: { width: px(1), style: "solid", color: tokens.color.brand },
    transition: {
      property: ["background-color", "translate"],
      duration: tokens.duration.fast,
      ease: tokens.ease.out,
    },
    on: {
      hover: { color: alpha(tokens.color.brand, percent(70)) },
      breakpoint: { md: { width: "100%" } },
    },
  },
});
```

Use named imports from `alien` and `alien/values`; aliases of those named imports are supported. Calls must be direct top-level variable initializers or direct default exports. Calls inside functions, namespace imports used for creation, and aliases of `alien` or `alien.create` fail during compilation instead of surviving into runtime.

Style objects and values must be inline, statically recognized expressions. Local variable substitution, imported constants, object spreads, destructuring helpers and dynamic values are not implemented. Simple finite numeric arithmetic inside helpers is supported, e.g. `px(4 * 5)`. Unsupported expressions receive diagnostics; TypeScript alone cannot enforce these syntactic restrictions.

## Values

Length helpers: `px`, `rem`, `em`, `ch`, `vw`, `vh`, `svh`, `dvh`. Compatible dimensional strings such as `"12px"`, `"1.5rem"`, and `"100%"` are accepted in dimensional positions. `percent`, `fr`, and `ms` retain distinct types. The currently typed string surface focuses on length-or-percentage dimensions; use helpers for border widths and other length-only fields.

`calc.sub()` currently subtracts two lengths. General calculations, percentage calculations and token arithmetic are future work. Numeric helper arguments support constant arithmetic, but do not evaluate arbitrary JavaScript. Length and color expressions reject nesting beyond 64 levels with a compiler diagnostic.

Color helpers: `hex`, `rgb`, `oklch`, `hsl`, `hwb`, `hsv`, `alpha`, `mix`. HSV helpers are converted to RGB. `mix` uses the OKLCH interpolation space and its percentage describes the second color. `alpha` sets, rather than multiplies, opacity; the outermost call wins. General alpha expressions emit CSS relative colors and mixing emits `color-mix`, so consumers must target browsers supporting these features. Nonstandard string functions such as `hsv()` are rejected; use the helper instead.

`custom` remains supported in free mode for compatibility, but most free-mode literal values no longer require it. It cannot bypass strict token policies. Custom font family names and animation identifiers remain supported because they do not currently have token registries.

## Conditions and composition

`on` accepts the documented simple pseudo-classes and built-in minimum-width breakpoints. Conditions can combine through nesting. Repeated pseudo-classes, nested breakpoints, unknown breakpoint names (including empty blocks), duplicate declarations and overlapping shorthand/longhand assignments within one condition are rejected.

Rules are sorted by pseudo-class specificity, then ascending breakpoint width, then a fixed pseudo-class order. At the same breakpoint, hover precedes focus and active. Explicit intersections have greater specificity. Object key order does not select a winner between these conditions. A general ambiguity checker for intersecting conditions is not implemented.

`border`, `transition`, and `animation` emit individual provided longhands. Missing fields do not reset other fields. A transition property list shares one duration, delay and easing; independent timing tracks are not implemented.

`cx()` joins classes and ignores `false`, `null`, `undefined` and empty strings. Argument order does not override CSS cascade order. There is no variants or composition-merging API.

## Tooling

The Vite plugin transforms JavaScript/JSX and TypeScript/TSX modules, Vue/Svelte script blocks, and Astro frontmatter. It runs before framework transforms, skips declaration files, dependency files and query-bearing module IDs, emits source maps, caches unchanged source, and updates virtual stylesheets during development. Configuration changes regenerate types and trigger a full reload; additions and deletions are handled too.

Class names use a stable 128-bit SHA-256 prefix derived from the project namespace and module path, followed by a per-module index. The plugin detects prefix conflicts among modules it observes. Independently built projects sharing CSS must use distinct namespaces. The low-level Rust `compile_styles` and `compile_typescript` helpers still use local `a0`, `a1`, ... names: their results are isolated outputs. Use `transform_with_config` / `transform_typescript` with unique module IDs when combining modules.

## CSS optimization

The compiler compares grouped output with a hybrid of grouped rules and shared atomic declarations. It accepts a change only when the combined uncompressed CSS and JavaScript class map is smaller. It shares unconditional declarations only when no other declaration in the same style block conflicts with that property, and preserves grouped output otherwise. Adjacent identical styles can share a class.

Optimization is local to an `alien.create()` block. The search ranks repeated declarations by potential byte savings and considers up to 64 safe candidates. It explores incremental additions starting from grouped rules and incremental removals starting from all candidates. This second direction can find profitable combinations even when adding their members individually would increase size.

Declarations are resolved once. An exact cost model counts emitted UTF-8 CSS and JavaScript class-map bytes, including selectors, media queries, class separators, escaped style names and classes removed when a style becomes fully atomic. Candidate comparisons do not repeatedly allocate generated CSS. The original grouped output (including adjacent-style deduplication) remains the fallback, so optimization never increases this combined size. Results are deterministic for the same input and module identifier.

The optimizer does not compare gzip or Brotli sizes, include class occurrences in rendered HTML, or guarantee a global minimum. It does not share conditional declarations or deduplicate non-adjacent rules across potentially conflicting styles. Tests cover exact cost/emission agreement, shorthand and conditional conflicts, `!important`, empty styles, long namespaces, and the 64-candidate boundary. Run `bun run bench:compiler` for reproducible transform timing and output-size measurements.

## Framework integrations

See the [framework guide](./docs/frameworks.md) for the tested versions and exact setup for Vite, webpack, Next.js, Nuxt, Astro, SvelteKit and Angular. `alien generate` provides a bundler-independent path that emits class-map modules and `.alien/styles.css`; components import the generated modules, never the raw definitions.
