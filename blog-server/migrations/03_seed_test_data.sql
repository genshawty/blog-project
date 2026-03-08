-- Seed test data: 1 user + 3 posts
-- Password for test user: testpassword123

INSERT INTO users (id, username, email, password_hash)
VALUES (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11',
    'testuser',
    'testuser@example.com',
    '$argon2id$v=19$m=19456,t=2,p=1$hGGMQ3g4riSwR+78rigUaA$Htg47sXRPQ834BBAvN649MTisCNcu6wXtI239iTsmfM'
) ON CONFLICT DO NOTHING;

INSERT INTO posts (id, title, content, author_id)
VALUES
    (
        'b1eebc99-9c0b-4ef8-bb6d-6bb9bd380a01',
        'Getting Started with Rust',
        'Rust is a systems programming language that runs blazingly fast, prevents segfaults, and guarantees thread safety. In this post we explore the basics of ownership, borrowing, and lifetimes.',
        'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11'
    ),
    (
        'b1eebc99-9c0b-4ef8-bb6d-6bb9bd380a02',
        'Building Web APIs with Actix',
        'Actix-web is a powerful, pragmatic, and extremely fast web framework for Rust. Let us walk through building a RESTful API with routing, middleware, and database integration.',
        'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11'
    ),
    (
        'b1eebc99-9c0b-4ef8-bb6d-6bb9bd380a03',
        'Understanding gRPC in Rust',
        'gRPC provides a modern, high-performance RPC framework. Combined with tonic in Rust, it makes building efficient microservices straightforward. This post covers proto definitions, server setup, and client usage.',
        'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11'
    )
ON CONFLICT DO NOTHING;
