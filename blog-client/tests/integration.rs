use blog_client::{BlogClient, Transport};
use uuid::Uuid;

fn http_addr() -> String {
    std::env::var("BLOG_HTTP_ADDR").unwrap_or_else(|_| "http://localhost:8080".into())
}

fn grpc_addr() -> String {
    std::env::var("BLOG_GRPC_ADDR").unwrap_or_else(|_| "http://localhost:50051".into())
}

async fn new_client_like(c: &BlogClient) -> BlogClient {
    BlogClient::new(c.transport_kind(), c.transport_addr()).await
}

fn unique_user() -> (String, String) {
    let name = format!("user_{}", Uuid::new_v4());
    let email = format!("{}@test.com", name);
    (name, email)
}

/// Generates two #[ignore] tests from a single test function: `{name}_http` and `{name}_grpc`.
/// Each one creates a client with the corresponding transport and runs the same scenario.
/// `paste::paste!` concatenates the test name with the suffix (_http / _grpc).
macro_rules! transport_test {
    ($name:ident, $test_fn:ident) => {
        paste::paste! {
            #[tokio::test]
            #[ignore] // run manually: cargo test -- --ignored
            async fn [<$name _http>]() {
                let mut client = BlogClient::new(Transport::Http, &http_addr()).await;
                $test_fn(&mut client).await;
            }

            #[tokio::test]
            #[ignore]
            async fn [<$name _grpc>]() {
                let mut client = BlogClient::new(Transport::Grpc, &grpc_addr()).await;
                $test_fn(&mut client).await;
            }
        }
    };
}

// ============================================================
// Auth tests
// ============================================================

async fn test_register(c: &mut BlogClient) {
    let (user, email) = unique_user();
    let resp = c.register(&user, &email, "password123").await.unwrap();
    assert!(!resp.token.is_empty(), "token should not be empty");
    assert_eq!(resp.user.username, user);
}

async fn test_register_duplicate(c: &mut BlogClient) {
    let (user, email) = unique_user();
    c.register(&user, &email, "password123").await.unwrap();

    let mut c2 = new_client_like(c).await;
    let err = c2.register(&user, &email, "password123").await;
    assert!(err.is_err(), "duplicate registration should fail");
}

async fn test_login(c: &mut BlogClient) {
    let (user, email) = unique_user();
    c.register(&user, &email, "pass").await.unwrap();

    let mut c2 = new_client_like(c).await;
    let resp = c2.login(&user, "pass").await.unwrap();
    assert!(!resp.token.is_empty());
}

async fn test_login_wrong_password(c: &mut BlogClient) {
    let (user, email) = unique_user();
    c.register(&user, &email, "pass").await.unwrap();

    let mut c2 = new_client_like(c).await;
    let err = c2.login(&user, "wrong").await;
    assert!(err.is_err(), "wrong password should fail");
}

// ============================================================
// Post CRUD tests
// ============================================================

async fn test_create_post(c: &mut BlogClient) {
    let (user, email) = unique_user();
    c.register(&user, &email, "pass").await.unwrap();

    let post = c.create_post("Test Title", "Test Content").await.unwrap();
    assert!(!post.id.is_empty());
    assert_eq!(post.content, "Test Content");
}

async fn test_get_post(c: &mut BlogClient) {
    let (user, email) = unique_user();
    c.register(&user, &email, "pass").await.unwrap();

    let created = c.create_post("Title", "Content").await.unwrap();
    let fetched = c.get_post(&created.id).await.unwrap();
    assert_eq!(fetched.id, created.id);
    assert_eq!(fetched.content, "Content");
}

async fn test_get_post_not_found(c: &mut BlogClient) {
    let err = c.get_post("nonexistent-id-000").await;
    assert!(err.is_err(), "getting non-existent post should fail");
}

async fn test_update_post(c: &mut BlogClient) {
    let (user, email) = unique_user();
    c.register(&user, &email, "pass").await.unwrap();

    let post = c.create_post("Old Title", "Old Content").await.unwrap();
    let updated = c
        .update_post(&post.id, "New Title", "New Content")
        .await
        .unwrap();
    assert_eq!(updated.id, post.id);
    assert_eq!(updated.content, "New Content");
}

async fn test_delete_post(c: &mut BlogClient) {
    let (user, email) = unique_user();
    c.register(&user, &email, "pass").await.unwrap();

    let post = c.create_post("To Delete", "content").await.unwrap();
    c.delete_post(&post.id).await.unwrap();

    let err = c.get_post(&post.id).await;
    assert!(err.is_err(), "deleted post should not be found");
}

async fn test_delete_post_not_found(c: &mut BlogClient) {
    let (user, email) = unique_user();
    c.register(&user, &email, "pass").await.unwrap();

    let err = c.delete_post("nonexistent-id-000").await;
    assert!(err.is_err(), "deleting non-existent post should fail");
}

async fn test_list_posts(c: &mut BlogClient) {
    let (user, email) = unique_user();
    c.register(&user, &email, "pass").await.unwrap();

    // Create a few posts
    c.create_post("Post 1", "content 1").await.unwrap();
    c.create_post("Post 2", "content 2").await.unwrap();
    c.create_post("Post 3", "content 3").await.unwrap();

    let list = c.list_posts(10, 0).await.unwrap();
    assert!(list.posts.len() >= 3, "should have at least 3 posts");
    assert!(list.total >= 3);
}

async fn test_list_posts_pagination(c: &mut BlogClient) {
    let (user, email) = unique_user();
    c.register(&user, &email, "pass").await.unwrap();

    for i in 0..5 {
        c.create_post(&format!("Pag {i}"), &format!("content {i}"))
            .await
            .unwrap();
    }

    let page1 = c.list_posts(2, 0).await.unwrap();
    assert_eq!(page1.posts.len(), 2);
    assert_eq!(page1.limit, 2);
    assert_eq!(page1.offset, 0);

    let page2 = c.list_posts(2, 2).await.unwrap();
    assert_eq!(page2.posts.len(), 2);
    assert_eq!(page2.offset, 2);
}

// ============================================================
// Authorization / forbidden tests
// ============================================================

async fn test_create_post_unauthorized(c: &mut BlogClient) {
    // No token set
    let err = c.create_post("t", "c").await;
    assert!(err.is_err(), "creating post without auth should fail");
}

async fn test_update_post_forbidden(c: &mut BlogClient) {
    // User A creates a post
    let (user_a, email_a) = unique_user();
    c.register(&user_a, &email_a, "pass").await.unwrap();
    let post = c.create_post("A's post", "content").await.unwrap();

    // User B tries to update it
    let (user_b, email_b) = unique_user();
    c.register(&user_b, &email_b, "pass").await.unwrap();

    let err = c.update_post(&post.id, "hacked", "hacked").await;
    assert!(err.is_err(), "updating another user's post should fail");
}

async fn test_delete_post_forbidden(c: &mut BlogClient) {
    // User A creates a post
    let (user_a, email_a) = unique_user();
    c.register(&user_a, &email_a, "pass").await.unwrap();
    let post = c.create_post("A's post", "content").await.unwrap();

    // User B tries to delete it
    let (user_b, email_b) = unique_user();
    c.register(&user_b, &email_b, "pass").await.unwrap();

    let err = c.delete_post(&post.id).await;
    assert!(err.is_err(), "deleting another user's post should fail");
}

// ============================================================
// Full CRUD flow
// ============================================================

async fn test_full_crud_flow(c: &mut BlogClient) {
    let (user, email) = unique_user();

    // Register
    let auth = c.register(&user, &email, "pass").await.unwrap();
    assert!(!auth.token.is_empty());

    // Create
    let post = c.create_post("Flow Title", "Flow Content").await.unwrap();
    assert_eq!(post.content, "Flow Content");

    // Read
    let fetched = c.get_post(&post.id).await.unwrap();
    assert_eq!(fetched.id, post.id);

    // Update
    let updated = c
        .update_post(&post.id, "Updated", "Updated Content")
        .await
        .unwrap();
    assert_eq!(updated.content, "Updated Content");

    // List (should contain our post)
    let list = c.list_posts(100, 0).await.unwrap();
    assert!(list.posts.iter().any(|p| p.id == post.id));

    // Delete
    c.delete_post(&post.id).await.unwrap();

    // Verify deleted
    let err = c.get_post(&post.id).await;
    assert!(err.is_err());
}

// ============================================================
// Register all tests for both transports
// ============================================================

// Auth
transport_test!(test_register, test_register);
transport_test!(test_register_duplicate, test_register_duplicate);
transport_test!(test_login, test_login);
transport_test!(test_login_wrong_password, test_login_wrong_password);

// CRUD
transport_test!(test_create_post, test_create_post);
transport_test!(test_get_post, test_get_post);
transport_test!(test_get_post_not_found, test_get_post_not_found);
transport_test!(test_update_post, test_update_post);
transport_test!(test_delete_post, test_delete_post);
transport_test!(test_delete_post_not_found, test_delete_post_not_found);
transport_test!(test_list_posts, test_list_posts);
transport_test!(test_list_posts_pagination, test_list_posts_pagination);

// Auth/Forbidden
transport_test!(test_create_post_unauthorized, test_create_post_unauthorized);
transport_test!(test_update_post_forbidden, test_update_post_forbidden);
transport_test!(test_delete_post_forbidden, test_delete_post_forbidden);

// Full flow
transport_test!(test_full_crud_flow, test_full_crud_flow);
