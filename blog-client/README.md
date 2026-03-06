# blog-client

A Rust client library for the blog API. Supports both HTTP (REST) and gRPC transports through a unified `BlogApi` trait.

## Usage

```rust
use blog_client::{BlogClient, Transport};

let mut client = BlogClient::new(Transport::Http, "http://localhost:8080").await;
let auth = client.register("user", "user@example.com", "password").await?;
let post = client.create_post("Title", "Content").await?;
```

## Integration tests

Tests are marked `#[ignore]` and require a running server.

```bash
# 1. Start the server (from the project root, using the Makefile):
make test-server

# 2. In another terminal, run the tests:
make test-client
```

Or run directly with cargo:

```bash
cargo test -p blog-client -- --ignored
```

You can override addresses via environment variables:

```bash
BLOG_HTTP_ADDR=http://localhost:9090 BLOG_GRPC_ADDR=http://localhost:50052 \
  cargo test -p blog-client -- --ignored
```

> The project root Makefile (`../Makefile`) has `test-server` and `test-client` targets that wire up all the environment variables automatically.
