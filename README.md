# Blog Platform

A blog platform with a Rust backend (Actix-web + gRPC), client SDK, and CLI.

## Structure

- `blog-server/` — REST + gRPC API server (auth, posts CRUD)
- `blog-client/` — Client SDK library (HTTP and gRPC transport)
- `blog-cli/` — Command-line interface for the API
- `blog-wasm/` — WASM frontend

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `HOST` | `127.0.0.1` | Server bind address |
| `PORT` | `8080` | HTTP port |
| `GRPC_PORT` | `50051` | gRPC port |
| `JWT_SECRET` | *(required)* | Secret key for JWT token signing |
| `CORS_ORIGINS` | `*` | Comma-separated allowed origins (`*` for any) |
| `DATABASE_URL` | — | PostgreSQL connection string, required for `postgres` mode (e.g. `postgres://user:pass@localhost:5432/blog`) |

## Running

```bash
# In-memory storage (default)
make test-server

# PostgreSQL storage
make test-server-pg DATABASE_URL=postgres://user:pass@localhost:5432/blog
```

Override any variable: `make test-server PORT=3000 JWT_SECRET=my-secret`

## Testing

Start the server first, then run tests in a separate terminal:

| Command | Description |
|---------|-------------|
| `make test-server` | Start the server with in-memory storage |
| `make test-server-pg` | Start the server with PostgreSQL storage |
| `make test-client` | Run blog-client integration tests |
| `make test-cli` | Run all CLI commands sequentially (register, login, CRUD via HTTP and gRPC) |

## API

| Method | Endpoint              | Auth | Description       |
|--------|-----------------------|------|-------------------|
| POST   | /api/auth/register    | No   | Register user     |
| POST   | /api/auth/login       | No   | Login             |
| GET    | /api/posts            | No   | List posts        |
| GET    | /api/posts/{id}       | No   | Get post          |
| POST   | /api/posts            | Yes  | Create post       |
| PUT    | /api/posts/{id}       | Yes  | Update post       |
| DELETE | /api/posts/{id}       | Yes  | Delete post       |

## TODO

- [ ] Unit tests for domain logic (post validation, user creation)
- [ ] Unit tests for services (BlogService, AuthService)
- [ ] Integration tests for API handlers
- [ ] JWT middleware tests (valid/invalid/expired tokens)
- [x] Replace in-memory repositories with database (PostgreSQL + sqlx)
- [x] Add database migrations
- [x] Connection pooling and repository trait implementations for DB
- [ ] Extract shared DTOs into `blog-common` crate (avoid type duplication between server and client)
- [ ] End-to-end integration tests for blog-server (HTTP + gRPC endpoints, auth flow, CRUD lifecycle)
