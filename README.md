# elijahncouch.github.io

My portfolio — a **Rust app compiled to WebAssembly** with [Dioxus](https://dioxuslabs.com).
The whole page (layout, content, theme toggle, and the interactive demos — BLE scanner,
sorting visualizer, and the `air` encryption demo) is written in Rust and runs as WASM.

## Layout

- Root (`index.html`, `assets/`, `wasm/`) — the built site GitHub Pages serves.
- `app/` — the Rust source.

## Build

```sh
cd app
dx build --release --platform web
# output lands in target/dx/portfolio/release/web/public
# copy its contents to the repo root, then commit
```

Requires the `wasm32-unknown-unknown` target and the Dioxus CLI (`dx`). Pinned to
wasm-bindgen 0.2.99 / web-sys 0.3.76 to match the bundled `dx` bindgen.
