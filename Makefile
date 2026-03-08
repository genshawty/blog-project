.PHONY: test-server test-server-pg client test-client test-cli docker-server docker-test docker-fullstack docker-down

# Environment variables:
#   HOST         - server bind address (default: 127.0.0.1)
#   PORT         - HTTP port (default: 8080)
#   GRPC_PORT    - gRPC port (default: 50051)
#   JWT_SECRET   - secret key for JWT token signing (required)
#   CORS_ORIGINS - comma-separated allowed origins, "*" for any (default: *)
#   DATABASE_URL - PostgreSQL connection string, required for "postgres" mode
#                  (e.g. postgres://user:pass@localhost:5432/blog)

HOST ?= 127.0.0.1
PORT ?= 8080
GRPC_PORT ?= 50051
JWT_SECRET ?= dev-secret-key-change-in-production
CORS_ORIGINS ?= *
DATABASE_URL ?= postgres://postgres:postgres@localhost:5432/blog

# Run server with in-memory storage
test-server:
	HOST=$(HOST) PORT=$(PORT) GRPC_PORT=$(GRPC_PORT) JWT_SECRET=$(JWT_SECRET) CORS_ORIGINS=$(CORS_ORIGINS) \
		cargo run -p blog-server

# Run server with PostgreSQL storage (requires DATABASE_URL)
test-server-pg:
	HOST=$(HOST) PORT=$(PORT) GRPC_PORT=$(GRPC_PORT) JWT_SECRET=$(JWT_SECRET) CORS_ORIGINS=$(CORS_ORIGINS) \
		DATABASE_URL=$(DATABASE_URL) cargo run -p blog-server -- postgres

client:
	cargo run -p blog-client

test-client:
	BLOG_HTTP_ADDR=http://$(HOST):$(PORT) BLOG_GRPC_ADDR=http://$(HOST):$(GRPC_PORT) \
		cargo test -p blog-client -- --ignored

CLI = cargo run -p blog-cli --

# Runs all CLI commands sequentially; greps post ID from create output to use in get/update/delete
test-cli:
	@echo "=== 1. Register ===" && \
	$(CLI) register --username "testuser" --email "test@example.com" --password "password123" && \
	echo "=== 2. Login ===" && \
	$(CLI) login --username "testuser" --password "password123" && \
	echo "=== 3. Create post ===" && \
	POST_ID=$$($(CLI) create --title "First post" --content "Hello world" | grep "ID:" | head -1 | awk '{print $$2}') && \
	echo "Created post: $$POST_ID" && \
	echo "=== 4. List posts ===" && \
	$(CLI) list --limit 10 --offset 0 && \
	echo "=== 5. Get post ===" && \
	$(CLI) get --id $$POST_ID && \
	echo "=== 6. Update post ===" && \
	$(CLI) update --id $$POST_ID --title "Updated title" --content "Updated content" && \
	echo "=== 7. Delete post ===" && \
	$(CLI) delete --id $$POST_ID && \
	echo "=== 8. List posts ===" && \
	$(CLI) list && \
	echo "=== 9. Create post (gRPC) ===" && \
	GRPC_POST_ID=$$($(CLI) --grpc create --title "gRPC post" --content "Created via gRPC" | grep "ID:" | head -1 | awk '{print $$2}') && \
	echo "Created gRPC post: $$GRPC_POST_ID" && \
	echo "=== 10. List posts (gRPC) ===" && \
	$(CLI) --grpc list

# Docker commands
docker-server:
	docker compose up --build --force-recreate --remove-orphans server

docker-test:
	docker compose up --build --force-recreate --remove-orphans --abort-on-container-exit test

docker-fullstack:
	docker compose up --build --force-recreate --remove-orphans server frontend

docker-down:
	docker compose down --volumes
