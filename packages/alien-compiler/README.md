# Alien CSS compiler

Native compiler and portable generator for [Alien CSS](https://github.com/snowly14/alien#readme).

```sh
bun add @alien-css/core
bun add -d @alien-css/compiler
bun run alien generate
```

Import the generated `.alien.generated` modules, compose their styles using `cx` from `@alien-css/core/cx`, and include `.alien/styles.css`. Use `alien generate --standard` for string class maps. The compiler supports macOS, Windows, and Linux on x64 and arm64 (glibc and musl on Linux); Node.js 22 or newer is required.
