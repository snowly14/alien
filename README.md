# Alien CSS

Typed TypeScript styles compiled to CSS by Rust, with Vite/webpack integrations, portable generation, and built-in design tokens. Style creation runs at build time. The emitted application uses CSS class names; `cx()` is a small optional runtime helper.

## Develop

Requirements: Bun 1.4.3, Node.js 26.11.1 for the framework test matrix, and Rust 1.99.0 (pinned in `rust-toolchain.toml`). The Rust workspace also checks compatibility with Rust 1.97.0 in CI.

```sh
bun install --frozen-lockfile
bun run check          # Build packages/native binding, generate demo types, check Rust and TS
bun run lint           # rustfmt, Clippy, Biome
bun run test           # Rust, runtime, type, compiler, Vite build and HMR tests
bun run test:package   # Pack, install in a temporary project, prepare types, build with Vite
bun run test:frameworks # Astro, Nuxt, SvelteKit, Next webpack/Turbopack, Angular AOT
bun run demo
```

`bun run format` formats Rust and TypeScript. `bun run demo:build` builds the React example. `bun run bench:compiler` builds the native compiler in release mode and benchmarks single-style modules and blocks with 100–1,000 shared or unique styles. It reports timings and emitted CSS/class-map bytes; it is not an application performance guarantee.

## Framework support

See the [framework guide and verified matrix](./docs/frameworks.md) for React, Vue, Svelte, Solid, Astro, Nuxt, SvelteKit, Next.js and Angular. `alien-compiler` is independent of a bundler; `alien-vite` and `alien-webpack` integrate it, while `alien generate` emits ordinary modules and CSS for Angular, Turbopack and other build systems. Each route requires the configuration documented in the guide.

## Two policies, one compiler

- **Free** (default): tokens, typed helpers, supported CSS color strings and dimensional strings are accepted.
- **Strict**: colors, spacing, corner radius, font weights, font sizes, line heights and timing values use tokens. CSS keywords remain available. Dimensions such as width, height and border width still accept compatible literal values.

Strict mode checks both TypeScript and the compiler. `custom()` cannot bypass these token families. Put custom values in `alien.globals.ts` instead:

```ts
import { alien } from "alien";
import { hex, rem } from "alien/values";

export default alien.globals({
  mode: "strict",
  color: { brand: hex("#168043") },
  space: { control: rem(0.625) },
  radius: { control: rem(0.625) },
});
```

Then use the same imports as usual:

```ts
import { alien, tokens } from "alien";

export const styles = alien.create({
  button: {
    display: "flex",
    color: tokens.color.brand,
    padding: tokens.space.control,
    borderRadius: tokens.radius.control,
    on: { hover: { color: tokens.color.green[700] } },
  },
});
```

The configuration is parsed statically, never executed. The plugin refreshes `alien-env.d.ts` and invalidates compiled styles when configuration changes. Do not import the configuration into application code.

## Optional CSS reset

Set `reset: "light"`, `"medium"`, or `"strong"` in `alien.globals.ts`. Omit the setting or use `reset: false` to disable it. The Vite plugin adds the stylesheet to HTML entries, even for pages without Alien style calls. The reset is scoped to the `alien-reset` cascade layer so normal unlayered app styles take precedence.

See the [reset levels](./syntaxe.md#optional-css-reset) for the rules each setting applies.

## Integrate with Vite

```ts
import alien from "alien-vite";
import { defineConfig } from "vite";

export default defineConfig({ plugins: [alien()] });
```

Include `alien-env.d.ts` in the application's `tsconfig.json`. Generate it **before** type checking, including on a clean checkout:

```json
{
  "scripts": {
    "check": "alien prepare && tsc --noEmit",
    "dev": "vite",
    "build": "alien prepare && tsc --noEmit && vite build"
  }
}
```

Run scripts with Bun. `bun run alien prepare [project-root]` generates types without starting Vite. In a monorepo, use one TypeScript project per application; do not combine multiple generated `AlienProject` declarations in one program.

The plugin accepts `root` (the project root when different from Vite’s root), `globals` (configuration path relative to the Alien project root) and `namespace` (stable CSS namespace). Give independently built applications different namespaces when they share a page. The default namespace is the package name, or the root directory name when no package name exists. The preparation CLI uses `alien.globals.ts` by default. For a custom path, use `bun run alien prepare [project-root] --globals design.tokens.ts`, matching the plugin's `globals` option.

The default configuration file is optional. An explicitly configured path must exist: a missing file fails preparation or the build instead of silently reverting to free mode.

## Build distributable packages

```sh
bun run build:packages
bun run build:native --release
bun run test:package
```

Packages expose compiled ESM and `.d.ts` files. A local native build produces a binary for the current host. On Linux, Alien detects glibc or musl and selects the corresponding binary.

The [Native packages workflow](./.github/workflows/native-packages.yml) builds and smoke-tests eight targets with Node and Bun. It covers macOS and Windows on x64 and ARM64, plus both Linux libc variants on both architectures. After every target passes, the workflow installs the assembled archives in a temporary project, then uploads the four npm tarballs as `alien-npm-packages`. Download this artifact to get a release candidate. Publication requires a registry name you control, since `alien` is already registered on npm.

The package smoke test normally rebuilds a debug native binary. To retain a release binary for packaging, run `bun run build:native --release` after the smoke test and before packing.

## CSS size comparison

Run `bun run bench:css` to compare Alien, Tailwind and StyleX on the same declarations. The [benchmark results](./docs/css-benchmark.md) include minified CSS, gzip and Brotli.

## Current scope

See [supported syntax](./syntaxe.md) for exact contracts and limitations. [The original proposal](./docs/proposals/initial-api.md) is retained separately and is not a promise that its APIs exist.

The Rust CLI emits CSS or validates an input file:

```sh
cargo run -p alien-cli -- examples/react/src/button.tsx --globals examples/react/alien.globals.ts --check
```

There is no theme system, variants API, keyframes builder, or arbitrary JavaScript evaluator yet. The compiler automatically combines grouped rules with shared atomic declarations when this reduces the combined raw CSS and class-map size. It resolves declarations once and compares bounded candidate plans using exact byte costs before emitting the chosen output. The optimizer works within each style block and keeps potentially conflicting declarations grouped. See the [optimization contract](./syntaxe.md#css-optimization) for its guarantees and limits.

## Optimization and measurements

Alien chooses between grouped declarations and shared classes using the combined size of CSS and generated class maps. Sharing respects shorthand conflicts, conditional overrides, importance, and source-order composition. Separate `alien.create()` calls in one module can share safe declarations; `alien generate` additionally optimizes the complete set of discovered style files.

See the [CSS size report](./docs/css-benchmark.md) and [complete-project integration](./docs/frameworks.md#complete-project-optimization) for measured results and tradeoffs. Run `bun run bench:css` to regenerate the English HTML/Markdown reports and inspect the exact CSS. Run `bun run bench:compiler` for native compilation timings and class-map sizes; compare timings on an otherwise idle machine.
