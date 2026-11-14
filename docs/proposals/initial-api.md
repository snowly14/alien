> Historical design proposal, not the implemented API. See ../../syntaxe.md for the supported syntax. Themes, variants, keyframes, overrides and atomic optimization below remain proposals.

# Alien syntax

The proposed API for a fully typed CSS framework designed for autocomplete and minimal string literals. The contracts below define the intended behavior of the compiler, generated project types, and runtime helpers.

## Syntax rules

1. **Use the same names everywhere.** Use `color`, `radius`, and `space` in both `globals` and `tokens`.
2. **Group CSS keywords by property.** `css.<property>.<value>` makes examples such as `css.borderStyle.solid` and `css.display.flex` easy to find. Autocomplete shows the valid keywords for each property.
3. **Put conditions in `on`.** Use `on.hover` and `on.focusVisible` for interaction states, `on.theme.dark` for a declared theme, and `on.breakpoint.md` for a declared breakpoint.
4. **Prefer typed helpers and object keys.** Use `px()`, `rem()`, `ms()`, `percent()`, `alpha()`, and `calc` for values and calculations. A string can still be the input to a helper such as `hex()`.
5. **Declare themes once.** List them in `alien.globals()` and choose one as `defaultTheme`. Each themed token must provide a value for every declared theme. TypeScript reports any missing theme.
6. **Use objects for grouped CSS declarations.** Objects such as `border`, `transition`, and `animation` offer autocomplete for their fields. Each provided field updates its corresponding CSS property.
7. **Make exceptions explicit.** Component colors, spacing, radius, and font weights follow the token policy below. `custom(...)` explicitly authorizes a value outside that policy while preserving its value type.

---

## 1. Global configuration

```ts
// alien.globals.ts
import { alien } from "alien";
import { hex, oklch, rem } from "alien/values";

alien.override({
  color: { green: { 600: hex("#168043") } },
  radius: { lg: rem(0.625) },
});

alien.globals({
  themes: ["light", "dark", "mauve"],
  defaultTheme: "light",

  color: {
    muted: {
      light: alien.color.green[600],
      dark: hex("#a3a3a3"),
      mauve: oklch(0.6056, 0.2189, 292.72),
    },
  },
  radius: {
    card: rem(0.625),
  },
});
```

---

## 2. Styles

```ts
// styles.ts
import { alien, css, custom, tokens } from "alien";
import { alpha, calc, hex, percent, px, rem } from "alien/values";

export const s = alien.create({
  button: {
    padding: tokens.space.md,
    backgroundColor: tokens.color.green[600],
    color: tokens.color.gray[50],
    borderRadius: tokens.radius.lg,
    fontSize: px(14),
    minWidth: calc.sub(rem(12), px(5)),

    // Each field maps to its corresponding CSS property.
    border: {
      width: px(1),
      style: css.borderStyle.solid,
      color: tokens.color.green[700],
    },

    on: {
      hover: { backgroundColor: tokens.color.green[700] },
      focusVisible: {
        outlineWidth: px(2),
        outlineStyle: css.outlineStyle.solid,
        outlineColor: alpha(tokens.color.green[700], percent(40)),
      },
      theme: {
        dark: {
          backgroundColor: tokens.color.green[500],
          on: {
            hover: { backgroundColor: tokens.color.green[400] },
          },
        },
      },
    },
  },

  sidebar: {
    padding: tokens.space.md,
    color: tokens.color.muted,
    borderRadius: tokens.radius.card,
  },

  campaignBadge: {
    backgroundColor: custom(hex("#ff00aa")),
    fontWeight: custom(650),
  },

  featured: {
    fontWeight: css.fontWeight.bold,
  },
});
```

---

## 3. Transitions and animations

```ts
import { alien, css, tokens } from "alien";
import { ms } from "alien/values";

const fadeIn = alien.keyframes({
  from: { opacity: 0 },
  to: { opacity: 1 },
});

export const interactive = alien.create({
  button: {
    backgroundColor: tokens.color.green[600],
    transition: {
      property: css.transitionProperty.backgroundColor,
      duration: ms(180),
      ease: tokens.ease.out,
    },
    on: {
      hover: { backgroundColor: tokens.color.green[700] },
    },
  },

  badge: {
    // Animation uses the same object syntax as transition.
    animation: {
      name: fadeIn,
      duration: ms(300),
      ease: tokens.ease.out,
    },
  },
});
```

---

## 4. Variants

```ts
import { alien } from "alien";
import { cx } from "alien/cx";
import { percent, rem } from "alien/values";
import { s } from "./styles";

// Variant names have their own namespace, so a variant can be named "defaults".
const buttonVariant = alien.variants({
  base: s.button,
  variants: {
    size: {
      xs: { minHeight: rem(2) },
      sm: { minHeight: rem(2.5) },
    },
    full: {
      true: { width: percent(100) },
      false: {},
    },
  },
  defaults: { size: "xs", full: false },
});

// The result includes the base and the selected variants.
const buttonClass = buttonVariant({ size: "sm", full: true });

// Compose additional classes when needed.
export function getButtonClass(featured: boolean) {
  return cx(buttonClass, featured && s.featured);
}
```

> Variant names stay readable as project-defined strings. TypeScript checks them against a union such as `"xs" | "sm"`.

---

## 5. Behavior contracts

### Token policy

The initial component policy applies to styles, conditional declarations, variants, and keyframes. Configuration declarations accept typed literal values to define the design system.

| Property family | Accepted values |
| --- | --- |
| Color-valued properties, including border and outline colors | Color tokens, transformations of color tokens, valid CSS keywords, or `custom(...)` |
| Padding, margin, and gaps | Space tokens, calculations derived from space tokens, valid CSS keywords, or `custom(...)` |
| Border radius | Radius tokens, calculations derived from radius tokens, valid CSS keywords, or `custom(...)` |
| Dimensions, font sizes, and border or outline widths | Compatible typed values, including `px()`, `rem()`, and `calc` |
| Font weights | Font weight tokens, valid CSS keywords, or `custom(...)` |
| Other properties | Values compatible with the property's type |

`alpha(tokens.color.green[700], percent(40))` retains its token origin and is valid as a component color. For a calculation to retain a token origin, each dimensional input must come from a compatible token. Dimensionless factors may be literal numbers. A space token multiplied by a number remains a space-derived value. Adding a free length requires an explicit exception when used for padding, margin, or gaps.

`custom(...)` preserves the type of its argument. A custom color assigned to padding is a type error. The compiler validates literal values, including hexadecimal syntax and alpha ranges, and identifies exception locations in diagnostics. A bare `hex()` result in a component color and a numeric literal for `fontWeight` require `custom(...)`.

### Theme maps and token references

The declared `themes` list determines the required keys of every theme map. `defaultTheme` must name one of these themes. Duplicate theme names produce a diagnostic.

For the initial color schema, each named entry is a typed color value, a palette with numeric shade keys, or a theme map. Each palette shade may itself be a typed color value or a theme map. Theme-map keys must match the declared themes exactly. A missing or misspelled theme produces a diagnostic. The generated project types derive the expected keys from `themes`, so an incomplete map cannot silently become a group of unrelated tokens.

`alien.color.green[600]` is a symbolic reference to the built-in token. The compiler applies the project's override before resolving references from global tokens. The example's `muted.light` therefore resolves to `hex("#168043")`. The same overridden value is available as `tokens.color.green[600]` in styles. Unspecified shades retain their built-in values.

`alien.override()` accepts existing built-in token paths with compatible value types. `alien.globals()` adds project token paths. Replacing an existing built-in leaf through `globals` produces a diagnostic directing the author to `override`. Duplicate assignments to the same token path also produce a diagnostic.

### Conditions and precedence

Interaction states, theme names, and breakpoint names have separate namespaces. Theme names come from `themes`. Breakpoint names must come from the project's effective breakpoint registry, including any built-in breakpoints. Unknown conditions produce diagnostics.

Nested conditions combine their requirements. `on.theme.dark.on.hover` applies when the dark theme is active and the element is hovered.

Within a style, the compiler preserves this priority from lowest to highest.

1. Base declarations.
2. Theme-only and breakpoint-only declarations.
3. Interaction-state declarations.
4. Explicit combinations of those conditions, with more constrained combinations taking precedence.

Breakpoints at the same specificity are emitted from smallest to largest. Simple pseudo-classes at the same breakpoint use a fixed order, with link states before hover, focus after hover, and active after focus. Use nested states such as `on.hover.on.active` to target their intersection.

If overlapping conditions at the same priority assign different values to the same property, the compiler requires an explicit combination or reports an ambiguity. Reordering object keys does not resolve the conflict. The emitted selectors and rule order must preserve this contract.

For the button example, the dark theme uses green 500 at rest and green 400 on hover. The other themes use green 600 at rest and green 700 on hover.

### Grouped declarations

Each object field updates its corresponding CSS property. `border: { color: value }` emits `border-color` and preserves the existing width and style. Omitted fields preserve their current CSS behavior.

Duplicate or overlapping property assignments within one condition produce a diagnostic. Examples include `border.color` together with `borderColor`, and `borderColor` together with `borderTopColor`. The compiler may combine longhands into a shorthand only when the resulting cascade and reset behavior are equivalent.

The same field-level behavior applies to `transition` and `animation`. Each entry in a `transition.property` list shares the same duration, delay, and easing. Multiple tracks with independent timing need an explicit representation.

### Transitions and class composition

The transition example explicitly targets `background-color`. Its value follows the relevant CSS property name through `css.transitionProperty.backgroundColor`.

If automatic transition inference is added, its contract must define which local conditions it inspects and which properties it emits. Named groups of transition properties likewise require a documented expansion.

`alien.variants()` includes `base` in its result. Selected variants override conflicting base declarations within the same condition. Conflicting assignments between independently selected variants require an explicit resolution or a compiler diagnostic. Variant defaults and call arguments are checked against the declared variant names and choices.

`cx()` joins class names and ignores absent conditional entries such as `false`, `null`, and `undefined`. Class arguments retain normal CSS cascade behavior. The order of arguments is not a style-override contract.

---

## 6. Compiler and tooling requirements

- **Project discovery.** Discover `alien.globals.ts` for the current project and statically collect its declarations. Resolve overrides before global token references, independently of declaration order. Report duplicate declarations and reference cycles.
- **Generated project types.** Generate declarations for custom token paths, theme names, and breakpoint names. Integrate them with TypeScript so styles can import `tokens` directly from `alien`. Scope declarations to their project to keep separate projects independent.
- **Editor updates.** Refresh generated types during development when the global file changes. Remove stale names when tokens or themes are deleted. Report invalid configuration at its source location.
- **Type checking.** Generate and validate project declarations before running the project's TypeScript check, including a clean checkout and CI. Missing or invalid generated types must produce an actionable error. Generated declarations must preserve precise types and diagnostics.
- **Static CSS generation.** Extract styles and resolve helpers at build time. Runtime helpers handle class composition and variant selection. Theme-dependent values use CSS custom properties whose values change under the active theme selector. Styles continue to reference the same token.
- **CSS optimization.** Emit the tokens, declarations, and keyframes required by the compiled styles. Compare grouped and atomic output by minified and compressed size. Preserve conditional behavior, variant precedence, and CSS cascade semantics during optimization.
- **TypeScript performance.** Measure editor responsiveness on projects with many tokens, properties, and themes. The public API must remain usable without noticeably expensive autocomplete.
- **Validation.** Verify generated types with valid and invalid examples. Check emitted CSS for all themes, combined conditions, partial grouped declarations, and variant combinations. Parsing the examples checks their syntax only. API typing and generated CSS require implementation-level verification.
