# Alien CSS

Typed TypeScript styles compiled to CSS at build time. For Vite:

```sh
bun add @alien-css/core
bun add -d @alien-css/vite
```

```ts
import { alien } from "@alien-css/core";

export const styles = alien.create({ button: { color: "red", padding: "1rem" } });
```

Configure `@alien-css/vite` before framework plugins. Portable generation uses `@alien-css/compiler` and `cx` from `@alien-css/core/cx`; see the [setup guide](https://github.com/snowly14/alien#readme) for usage, supported platforms, and composition semantics.
