use clap::Parser;
use dotenv::dotenv;
use zoo_message_primitives::zoo_utils::{
    encryption::string_to_encryption_static_key, signatures::string_to_signature_secret_key,
};
use zoo_libp2p_relayer::{LibP2PProxy, LibP2PRelayError};

#[derive(Parser, Debug)]
#[command(name = "Zoo LibP2P Relayer")]
#[command(author = "Zoo Team <team@zoo.ngo>")]
#[command(version, about = "Relays LibP2P connections for Zoo", long_about = None)]
struct Args {
    /// Sets the port to bind the server
    #[arg(short, long, default_value = "8080", env = "PORT")]
    port: u16,

    /// RPC URL for the registry
    #[arg(long, env = "RPC_URL")]
    rpc_url: Option<String>,

    /// Contract address for the registry
    #[arg(long, env = "CONTRACT_ADDRESS")]
    contract_address: Option<String>,

    /// Identity secret key
    #[arg(long, env = "IDENTITY_SECRET_KEY")]
    identity_secret_key: String,

    /// Encryption secret key
    #[arg(long, env = "ENCRYPTION_SECRET_KEY")]
    encryption_secret_key: String,

    /// Node name for registry
    #[arg(long, env = "NODE_NAME")]
    node_name: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), LibP2PRelayError> {
    dotenv().ok();

    let args = Args::parse();

    let identity_secret_key = string_to_signature_secret_key(&args.identity_secret_key)?;
    let encryption_secret_key = string_to_encryption_static_key(&args.encryption_secret_key)?;

    println!("Starting LibP2P Proxy Server on port {}", args.port);

    let proxy = LibP2PProxy::new(
        Some(identity_secret_key),
        Some(encryption_secret_key),
        args.node_name,
        args.rpc_url,
        args.contract_address,
        None, // max_connections
        Some(args.port),
    )
    .await?;

    proxy.start().await?;

    Ok(())
}
