pub mod blog {
    tonic::include_proto!("blog");
}

pub mod error;
pub mod grpc_client;
pub mod http_client;
