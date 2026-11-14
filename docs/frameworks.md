# Framework integration

Alien emits ordinary class names and CSS. The integration must compile style definitions **before** a framework turns components into JavaScript functions. Choose one integration path per source file; do not import both a `.alien.ts` definition and its generated module.

## Verified build matrix

These versions are installed in the repository lockfile. These are tested configurations, not a promise of compatibility with every version or third-party plugin.

| Framework / tool | Integration | Automated verification |
| --- | --- | --- |
| React 19.3 / Vite 8.3 | `alien-vite` before React plugin | JSX client and SSR builds; same class names and CSS |
| Solid 1.9 / Vite 8.3 | `alien-vite` before Solid plugin | JSX client and SSR builds; same class names and CSS |
| Vue 3.5 / Vite 8.3 | `alien-vite` before Vue plugin | `<script setup>` client/SSR builds and CSS hot updates |
| Svelte 5.57 / Vite 8.3 | `alien-vite` before Svelte plugin | Component scripts, client/SSR builds, distinct module/instance classes |
| SvelteKit 3.2 | `alien-vite` before `sveltekit()` | Static build, rendered classes, CSS and reset |
| Nuxt 4.6 | `alien-vite` with explicit project root | Client/server build and prerendered HTML/CSS |
| Astro 7.4 | `alien-vite` and `@astrojs/compiler` | Frontmatter compilation and static HTML/CSS |
| Next.js 16.4, webpack | `alien-webpack` pre-loader | App Router static export, rendered HTML/CSS |
| Next.js 16.4, Turbopack | Portable generation | Isolated installed project; App Router static export |
| Angular 22.2 | Portable generation | Actual Angular CLI AOT build with strict templates |
| webpack 5.112 | `alien-webpack` plus the host's CSS loaders | Real bundle and extracted stylesheet |

Run `bun run test` for compiler, integration, client/SSR and development-update tests. Run `bun run test:frameworks` for the framework CLI builds. The latter needs Node.js; CI uses Node 26.11.1. Angular's fixture requires TypeScript 6.0. The package smoke test separately checks declarations with TypeScript 5.9.

## Shared setup

Install `alien` as an application dependency, and the compiler integration as a development dependency. Build the local packages/native binary first when working in this repository. Published native binaries must match the build machine's OS and architecture; the runtime browser application does not load a native compiler.

Configuration remains in `alien.globals.ts`. Include `alien-env.d.ts` in the project's TypeScript configuration and run `alien prepare` before type checking. The strict/free policies, tokens, `radius`, and reset levels are identical across integrations. Vite/webpack optimize each module, including sharing across its `alien.create()` calls. Portable generation can additionally share safe declarations across all discovered style modules.

Use named ES imports from `alien` and `alien/values`. CommonJS `require()` style definitions are not supported. Calls must be direct top-level initializers; do not create styles inside a render function, hook, lifecycle handler or Angular class initializer.

## React, Solid and Vue with Vite

```sh
bun add alien
bun add -d alien-vite
```

```ts
import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue"; // or the project's React/Solid plugin
import alien from "alien-vite";

export default defineConfig({ plugins: [alien(), vue()] });
```

JS, JSX, TS, TSX, MJS, MTS and CTS source modules support the same static style syntax. Vue supports regular scripts and `<script setup>`, including TS/TSX. Import Alien in the same script block that declares the styles. For shared definitions, import an ordinary `.ts`/`.js` styles module from the script. External `<script src>` blocks are not part of the verified matrix. No regular expression is used to extract component scripts; Alien uses Vue's compiler parser.

```vue
<script setup lang="ts">
import { alien, tokens } from "alien";
const styles = alien.create({ button: { color: tokens.color.green[600] } });
</script>
<template><button :class="styles.button">Save</button></template>
```

## Svelte and SvelteKit

For a Svelte Vite application, use `plugins: [alien(), svelte()]`. For SvelteKit 3:

```ts
import { sveltekit } from "@sveltejs/kit/vite";
import adapter from "@sveltejs/adapter-static";
import alien from "alien-vite";

export default { plugins: [alien(), sveltekit({ adapter: adapter() })] };
```

Use the project's existing adapter. The static adapter above is the test fixture's choice, not an Alien requirement. SvelteKit 2's older configuration layout is not included in the verified matrix.

```svelte
<script lang="ts">
  import { alien, tokens } from "alien";
  const styles = alien.create({ button: { color: tokens.color.green[600] } });
</script>
<button class={styles.button}>Save</button>
```

Both instance and module scripts are processed before Svelte lowers them. Each script has its own stable class namespace; import the helpers in the script where they are used.

## Nuxt

Nuxt can use its `app/` directory as the Vite root. Pass the **project** root so Alien finds the same globals file and generates types in the expected location:

```ts
import { fileURLToPath } from "node:url";
import alien from "alien-vite";

export default defineNuxtConfig({
  vite: {
    plugins: [alien({ root: fileURLToPath(new URL(".", import.meta.url)) })],
  },
});
```

Vue script/setup styles work as above. Include the generated root `alien-env.d.ts` in the app's TypeScript project according to its existing Nuxt configuration.

## Astro

```sh
bun add -d alien-vite @astrojs/compiler
```

```ts
import { defineConfig } from "astro/config";
import alien from "alien-vite";
export default defineConfig({ vite: { plugins: [alien()] } });
```

```astro
---
import { alien, tokens } from "alien";
const styles = alien.create({ button: { color: tokens.color.green[600] } });
---
<button class={styles.button}>Save</button>
```

Frontmatter is compiled before Astro's transform. For browser scripts, declare styles in an imported `.ts`/`.js` module; inline browser `<script>` style calls inside `.astro` files are not handled. Vue/Svelte/React islands use their normal integration paths.

## Next.js with webpack (App Router)

```sh
bun add alien
bun add -d alien-compiler alien-webpack
```

```js
// next.config.mjs
import { alienRule } from "alien-webpack";
export default {
  webpack(config) {
    config.module.rules.unshift(alienRule({ root: process.cwd() }));
    return config;
  },
};
```

Use `next dev --webpack` / `next build --webpack`. Merge the rule into any existing webpack hook. Do not replace the framework's CSS loaders. The adapter writes content-addressed CSS under `.alien/css/` and imports it through Next's CSS pipeline. Configuration files are registered as build dependencies for rebuilds. A normal top-level definition works in a client or server component; `"use client"` directives are preserved.

This loader is **not** configured for Turbopack. The Pages Router's global CSS import restrictions also favor the portable generation path below, importing its stylesheet once from `_app`.

## Portable generation: Angular, Turbopack and other build systems

This route does not need a custom bundler hook. Source styles live in `*.alien.ts` (or `.js`, `.mts`, `.cts`, `.mjs`) files under `src/`:

```ts
// src/button.alien.ts
import { alien, tokens } from "alien";
export const styles = alien.create({ button: { color: tokens.color.green[600] } });
```

```sh
bun add alien
bun add -d alien-compiler
bun run alien generate
```

The command generates `src/button.alien.generated.ts` next to its source, plus `.alien/styles.css` and `alien-env.d.ts`. Import the generated module from components:

```ts
import { styles } from "./button.alien.generated";
```

Import `.alien/styles.css` once at the application root, or add it to the build tool's global stylesheet list. For Next App Router, import it from `app/layout.tsx`. For Angular, add `".alien/styles.css"` to `angular.json` → build options → `styles`, then bind `[class]="styles.button"` from a component field.

Define the normal production command as `alien generate && <framework build command>`, for example:

```json
{
  "scripts": {
    "build": "alien generate && ng build",
    "styles:watch": "alien generate --watch"
  }
}
```

For Next, replace `ng build` with `next build --turbopack`. Before starting development, run `alien generate` once, then run `styles:watch` alongside the framework dev server. The generator refreshes CSS/classes on style or globals changes, recovers after source errors, and removes its own obsolete modules after source deletion. It preserves the previous output when compilation fails and refuses to overwrite handwritten files.

Use `--source app` if style definitions are outside `src`, `--globals design.tokens.ts` for a custom config, and `--namespace my-app` for independently built applications sharing a page. Definitions retain their original source module identity. CSS is concatenated in source-path order. Runtime imports and their relative paths are retained; never import a raw `.alien.ts` definition from runtime code, including from another style definition.

Generated files belong in `.gitignore`:

```gitignore
.alien/
*.alien.generated.*
alien-env.d.ts
```

## Reset and server rendering

The Vite adapter imports the reset through the CSS pipeline whenever it emits styles, so it does not depend only on an HTML entry. For an app without any Alien styles, import `"alien-vite/reset.css"` from a root module to include the configured reset. HTML entries also receive it automatically. The generated stylesheet and webpack adapter include the same configured reset.

Client/server builds must use the same project root, namespace and globals. Class generation never depends on whether a build targets the browser or server. Actual deployment adapters, streaming behavior, every third-party plugin, and non-DOM renderers such as React Native are outside these tests. Alien produces CSS for web renderers.

## Integration references

The adapters use the documented [Vite plugin lifecycle](https://vite.dev/guide/api-plugin.html), [Vue SFC tooling](https://vuejs.org/api/sfc-tooling.html), [Svelte compiler API](https://svelte.dev/docs/svelte/svelte-compiler), and [webpack loader API](https://webpack.js.org/api/loaders/). Next's webpack and Turbopack configurations are separate: see [Next webpack configuration](https://nextjs.org/docs/app/api-reference/config/next-config-js/webpack) and [Turbopack loader limitations](https://nextjs.org/docs/app/api-reference/config/next-config-js/turbopack).

## Complete-project optimization

`alien generate` compiles the discovered `.alien.ts` / `.alien.js` files together. It preserves their sorted source order, checks conflicts across the complete stylesheet, and compares cross-module sharing against independently optimized groups. Generated modules use a common project namespace, reducing repeated namespace bytes in the compressed stylesheet.

Import the generated modules and load `.alien/styles.css` as described above. The generator updates all affected modules in watch mode. Deploy the generated modules and stylesheet from the same generation together: adding/removing files can change generated class names. The output contains all discovered styles; this route does not provide per-route CSS splitting or import-graph tree shaking.

The incremental Vite/webpack path keeps independent module namespaces and CSS chunks. It does not automatically use complete-project compilation. Prefer that path when route-level CSS splitting and component-local hot updates matter more than sharing across modules.

The native compiler exposes `transformProject([{ source, path, moduleId }])` for build integrations that own a complete, ordered set of style modules. It returns `{ modules, css }`; each module contains transformed code and UTF-16 source edits, while CSS is returned once at project level. Module identifiers must be unique, and every input must contain a style call. Call order determines cascade order.
