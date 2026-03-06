use blog_client::{BlogClient, Transport};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "blog-cli", about = "CLI for Blog API")]
struct Cli {
    /// Use gRPC transport instead of HTTP
    #[arg(long, global = true)]
    grpc: bool,

    /// Server address (default: localhost:8080 for HTTP, localhost:50051 for gRPC)
    #[arg(long, global = true)]
    server: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Register a new user
    Register {
        #[arg(long)]
        username: String,
        #[arg(long)]
        email: String,
        #[arg(long)]
        password: String,
    },
    /// Login with existing credentials
    Login {
        #[arg(long)]
        username: String,
        #[arg(long)]
        password: String,
    },
    /// Create a new post
    Create {
        #[arg(long)]
        title: String,
        #[arg(long)]
        content: String,
    },
    /// Get a post by ID
    Get {
        #[arg(long)]
        id: String,
    },
    /// Update a post
    Update {
        #[arg(long)]
        id: String,
        #[arg(long)]
        title: String,
        #[arg(long)]
        content: String,
    },
    /// Delete a post
    Delete {
        #[arg(long)]
        id: String,
    },
    /// List posts with pagination
    List {
        #[arg(long, default_value = "10")]
        limit: i64,
        #[arg(long, default_value = "0")]
        offset: i64,
    },
}

fn token_path() -> PathBuf {
    dirs_next().unwrap_or_else(|| PathBuf::from(".")).join(".blog_token")
}

fn dirs_next() -> Option<PathBuf> {
    std::env::current_dir().ok()
}

fn load_token() -> Option<String> {
    std::fs::read_to_string(token_path()).ok().map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
}

fn save_token(token: &str) {
    let _ = std::fs::write(token_path(), token);
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let transport = if cli.grpc {
        Transport::Grpc
    } else {
        Transport::Http
    };

    let default_addr = if cli.grpc {
        "http://localhost:50051"
    } else {
        "http://localhost:8080"
    };

    let addr = cli.server.as_deref().unwrap_or(default_addr);

    let mut client = BlogClient::new(transport, addr).await;

    if let Some(token) = load_token() {
        client.set_token(token);
    }

    let result = run_command(&mut client, cli.command).await;

    match result {
        Ok(()) => {}
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    }
}

async fn run_command(
    client: &mut BlogClient,
    command: Commands,
) -> Result<(), blog_client::error::BlogClientError> {
    match command {
        Commands::Register {
            username,
            email,
            password,
        } => {
            let resp = client.register(&username, &email, &password).await?;
            save_token(&resp.token);
            println!("Registered successfully!");
            println!("User: {} ({})", resp.user.username, resp.user.email);
            println!("Token saved to .blog_token");
        }
        Commands::Login { username, password } => {
            let resp = client.login(&username, &password).await?;
            save_token(&resp.token);
            println!("Logged in successfully!");
            println!("User: {} ({})", resp.user.username, resp.user.email);
            println!("Token saved to .blog_token");
        }
        Commands::Create { title, content } => {
            let post = client.create_post(&title, &content).await?;
            println!("Post created!");
            print_post(&post);
        }
        Commands::Get { id } => {
            let post = client.get_post(&id).await?;
            print_post(&post);
        }
        Commands::Update { id, title, content } => {
            let post = client.update_post(&id, &title, &content).await?;
            println!("Post updated!");
            print_post(&post);
        }
        Commands::Delete { id } => {
            client.delete_post(&id).await?;
            println!("Post {id} deleted.");
        }
        Commands::List { limit, offset } => {
            let resp = client.list_posts(limit, offset).await?;
            println!("Posts ({} total):", resp.total);
            for post in &resp.posts {
                println!("---");
                print_post(post);
            }
        }
    }
    Ok(())
}

fn print_post(post: &blog_client::PostResponse) {
    println!("  ID: {}", post.id);
    println!("  Title: {}", post.title);
    println!("  Content: {}", post.content);
    println!("  Author: {}", post.author_id);
    println!("  Created: {}", post.created_at);
    if let Some(updated) = &post.updated_at {
        println!("  Updated: {updated}");
    }
}
