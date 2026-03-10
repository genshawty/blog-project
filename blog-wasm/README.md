# blog-wasm

WASM frontend for the blog, built with egui.

## Configuring API URL

By default the frontend connects to `http://localhost:8080`.

To override, set the `API_BASE_URL` environment variable at build time:

```bash
API_BASE_URL=http://myserver:9090 trunk serve
```

```bash
API_BASE_URL=https://api.example.com trunk build --release
```
