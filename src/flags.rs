use clap::Subcommand;

#[derive(clap::Parser, Debug)]
#[command(version)]
pub struct Flags {
    /// Dash Platform server hostname or IPv4 address
    #[arg(short = 'i', long = "address")]
    pub server_address: String,

    /// Dash Core IP port
    #[arg(short = 'c', long)]
    pub core_port: u16,

    // Dash Core RPC user
    #[arg(short = 'u', long)]
    pub core_user: String,

    // Dash Core RPC password
    #[arg(short = 'p', long)]
    pub core_password: String,

    /// Dash Platform DAPI port
    #[arg(short = 'd', long)]
    pub platform_port: u16,
}


#[derive(Subcommand, Debug, Clone)]
enum Commands {
    Get(String),
    Set {
        key: String,
        value: String,
        is_true: bool
    },
    Help
}
