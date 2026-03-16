mod commands;

use clap::{Parser, Subcommand, ValueEnum};
use clap_complete::Shell;
use mog_core::error::MogError;
use mog_core::output::{OutputFormat, OutputRenderer};
use std::process;

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// mog — a cross-platform CLI for Microsoft 365 via Microsoft Graph
#[derive(Parser, Debug)]
#[command(name = "mog", version = VERSION, about = "A cross-platform CLI for Microsoft 365 via Microsoft Graph")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Profile to use
    #[arg(long, global = true, env = "MOG_PROFILE")]
    pub profile: Option<String>,

    /// Output as JSON
    #[arg(long, global = true)]
    pub json: bool,

    /// Output as stable plain text
    #[arg(long, global = true)]
    pub plain: bool,

    /// Output format
    #[arg(long, global = true, value_name = "FORMAT")]
    pub output: Option<String>,

    /// Never prompt; fail instead
    #[arg(long, global = true)]
    pub no_input: bool,

    /// Color output control
    #[arg(long, global = true, default_value = "auto")]
    pub color: ColorChoice,

    /// Verbose logging to stderr
    #[arg(long, global = true)]
    pub verbose: bool,

    /// Full debug output including raw HTTP
    #[arg(long, global = true)]
    pub debug: bool,

    /// JSON trace event per Graph call to stderr
    #[arg(long, global = true)]
    pub trace: bool,

    /// Graph API version
    #[arg(long, global = true, default_value = "v1.0", value_name = "VERSION")]
    pub api_version: String,

    /// Server-side $top parameter
    #[arg(long, global = true)]
    pub top: Option<u32>,

    /// Auto-paginate all results
    #[arg(long, global = true)]
    pub all: bool,

    /// Skip destructive-action confirmations
    #[arg(long, global = true)]
    pub force: bool,

    /// Override client ID
    #[arg(long, global = true)]
    pub client_id: Option<String>,

    /// Override tenant
    #[arg(long, global = true)]
    pub tenant: Option<String>,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum ColorChoice {
    Auto,
    Always,
    Never,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Authentication and profile management
    Auth {
        #[command(subcommand)]
        command: AuthCommands,
    },
    /// Mail operations
    Mail {
        #[command(subcommand)]
        command: MailCommands,
    },
    /// Calendar operations
    Calendar {
        #[command(subcommand)]
        command: CalendarCommands,
    },
    /// OneDrive file operations
    Files {
        #[command(subcommand)]
        command: FilesCommands,
    },
    /// Raw Microsoft Graph API call
    Graph {
        #[command(subcommand)]
        command: GraphCommands,
    },
    /// Configuration management
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },
    /// Generate shell completions
    Completions {
        /// Shell to generate completions for
        shell: Shell,
    },
    /// Show version information
    Version,
}

// === Auth Commands ===

#[derive(Subcommand, Debug)]
pub enum AuthCommands {
    /// Log in to Microsoft 365
    Login {
        /// Services to request (comma-separated: mail,calendar,files)
        #[arg(long, value_delimiter = ',')]
        services: Option<Vec<String>>,

        /// Request read-only permissions
        #[arg(long)]
        readonly: bool,

        /// Auth strategy override
        #[arg(long)]
        strategy: Option<String>,

        /// Auth type: user (delegated) or app (client credentials)
        #[arg(long, default_value = "user")]
        auth_type: String,

        /// Certificate file for client-credentials
        #[arg(long)]
        certificate_file: Option<String>,

        /// Certificate format override (pem or pfx)
        #[arg(long)]
        certificate_format: Option<String>,

        /// Read client secret from stdin
        #[arg(long)]
        client_secret_stdin: bool,

        /// Federated token file path
        #[arg(long)]
        federated_token_file: Option<String>,

        /// Additional scopes to add
        #[arg(long)]
        add_scopes: Option<String>,
    },
    /// Log out and clear cached tokens
    Logout,
    /// Show authentication status
    Status,
    /// Profile management
    Profile {
        #[command(subcommand)]
        command: ProfileCommands,
    },
    /// Explain permissions required for a command
    ExplainPermissions {
        /// Command to explain (e.g., "mail send")
        #[arg(required = true, num_args = 1..)]
        command: Vec<String>,
    },
    /// Generate admin consent URL
    AdminConsentUrl,
}

#[derive(Subcommand, Debug)]
pub enum ProfileCommands {
    /// List all profiles
    List,
    /// Create a new profile
    Create {
        /// Profile name
        name: String,
        /// Tenant ID or domain
        #[arg(long, required = true)]
        tenant: String,
        /// Client ID (optional, uses default)
        #[arg(long)]
        client_id: Option<String>,
        /// Auth strategy
        #[arg(long, default_value = "device-code")]
        strategy: String,
        /// Cloud environment
        #[arg(long, default_value = "public")]
        cloud: String,
        /// Scope bundles (comma-separated)
        #[arg(long, value_delimiter = ',')]
        scopes: Option<Vec<String>>,
    },
    /// Update an existing profile
    Update {
        /// Profile name
        name: String,
        /// Tenant ID or domain
        #[arg(long)]
        tenant: Option<String>,
        /// Client ID
        #[arg(long)]
        client_id: Option<String>,
        /// Auth strategy
        #[arg(long)]
        strategy: Option<String>,
        /// Cloud environment
        #[arg(long)]
        cloud: Option<String>,
    },
    /// Delete a profile
    Delete {
        /// Profile name
        name: String,
    },
    /// Set the default profile
    SetDefault {
        /// Profile name
        name: String,
    },
}

// === Mail Commands ===

#[derive(Subcommand, Debug)]
pub enum MailCommands {
    /// List mail messages (headers only by default)
    List {
        /// Show only unread messages
        #[arg(long)]
        unread: bool,
        /// Show messages since (e.g., 7d, 24h)
        #[arg(long)]
        since: Option<String>,
        /// Include full message body
        #[arg(long)]
        include_body: bool,
    },
    /// Search mail using KQL
    Search {
        /// KQL query string
        #[arg(long, required = true)]
        kql: String,
    },
    /// Read a full message
    Read {
        /// Message ID
        id: String,
        /// Body type preference (text or html)
        #[arg(long)]
        body_type: Option<String>,
    },
    /// Send a mail message
    Send {
        /// Recipient email addresses (repeatable)
        #[arg(long, required = true, num_args = 1..)]
        to: Vec<String>,
        /// Email subject
        #[arg(long, required = true)]
        subject: String,
        /// Body content file
        #[arg(long)]
        body_file: Option<String>,
        /// File attachments (repeatable)
        #[arg(long)]
        attach: Vec<String>,
    },
    /// Attachment operations
    Attachments {
        #[command(subcommand)]
        command: AttachmentCommands,
    },
}

#[derive(Subcommand, Debug)]
pub enum AttachmentCommands {
    /// List attachments for a message (metadata only)
    List {
        /// Message ID
        id: String,
    },
    /// Download attachments
    Download {
        /// Message ID
        id: String,
        /// Output directory
        #[arg(long)]
        out_dir: Option<String>,
        /// Specific attachment ID
        #[arg(long)]
        attachment_id: Option<String>,
        /// Output file path (with --attachment-id)
        #[arg(long)]
        out: Option<String>,
    },
}

// === Calendar Commands ===

#[derive(Subcommand, Debug)]
pub enum CalendarCommands {
    /// Today's events
    Today,
    /// This week's events
    Week,
    /// List events in a date range
    List {
        /// Start date (YYYY-MM-DD or ISO 8601)
        #[arg(long, required = true)]
        start: String,
        /// End date (YYYY-MM-DD or ISO 8601)
        #[arg(long, required = true)]
        end: String,
    },
    /// Create a new event
    Create {
        /// Event subject
        #[arg(long, required = true)]
        subject: String,
        /// Start datetime
        #[arg(long, required = true)]
        start: String,
        /// End datetime
        #[arg(long, required = true)]
        end: String,
        /// Attendee emails (comma-separated)
        #[arg(long, value_delimiter = ',')]
        attendees: Option<Vec<String>>,
    },
    /// Update an existing event
    Update {
        /// Event ID
        id: String,
        /// New subject
        #[arg(long)]
        subject: Option<String>,
        /// New start datetime
        #[arg(long)]
        start: Option<String>,
        /// New end datetime
        #[arg(long)]
        end: Option<String>,
    },
    /// Delete an event
    Delete {
        /// Event ID
        id: String,
    },
}

// === Files Commands ===

#[derive(Subcommand, Debug)]
pub enum FilesCommands {
    /// Search OneDrive files
    Search {
        /// Search query
        #[arg(long, short = 'q', required = true)]
        q: String,
    },
    /// Download a file
    Download {
        /// Item ID
        #[arg(long, required = true)]
        item_id: String,
        /// Output file path
        #[arg(long, required = true)]
        out: String,
    },
    /// Export/convert a file
    Export {
        /// Item ID
        #[arg(long, required = true)]
        item_id: String,
        /// Target format (e.g., pdf)
        #[arg(long, required = true)]
        format: String,
        /// Output file path
        #[arg(long, required = true)]
        out: String,
    },
    /// Upload a file
    Upload {
        /// Destination path in OneDrive
        #[arg(long, required = true)]
        dest: String,
        /// Local file path
        #[arg(long, required = true)]
        file: String,
        /// Use resumable upload
        #[arg(long)]
        resumable: bool,
        /// Chunk size in bytes (for resumable upload)
        #[arg(long)]
        chunk_size: Option<u64>,
    },
}

// === Graph Commands ===

#[derive(Subcommand, Debug)]
pub enum GraphCommands {
    /// Make a raw Graph API call
    Call {
        /// HTTP method (GET, POST, PATCH, PUT, DELETE)
        verb: String,
        /// Graph API path (e.g., /me/messages)
        path: String,
        /// Set $select query parameter
        #[arg(long)]
        select: Option<String>,
        /// Set $filter query parameter
        #[arg(long)]
        filter: Option<String>,
        /// Add custom header (repeatable, format: "Key: Value")
        #[arg(long)]
        header: Vec<String>,
        /// Request body from file
        #[arg(long)]
        body_file: Option<String>,
        /// Read request body from stdin
        #[arg(long)]
        body_stdin: bool,
    },
}

// === Config Commands ===

#[derive(Subcommand, Debug)]
pub enum ConfigCommands {
    /// Show current configuration
    Show,
    /// Set a configuration value
    Set {
        /// Configuration key (e.g., output.defaultFormat)
        key: String,
        /// Configuration value
        value: String,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Set up logging
    let log_level = if cli.debug {
        "debug"
    } else if cli.verbose {
        "info"
    } else {
        "warn"
    };

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(log_level))
        )
        .with_writer(std::io::stderr)
        .init();

    // Beta API warning
    if cli.api_version == "beta" {
        eprintln!("Warning: Using beta API. Endpoints may change without notice.");
    }

    let format = OutputFormat::from_flags(
        cli.json,
        cli.plain,
        cli.output.as_deref(),
    );

    let result = run_command(&cli, format).await;

    match result {
        Ok(()) => process::exit(0),
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(e.exit_code().code());
        }
    }
}

async fn run_command(cli: &Cli, format: OutputFormat) -> Result<(), MogError> {
    match &cli.command {
        Commands::Version => {
            let info = serde_json::json!({
                "version": VERSION,
                "platform": std::env::consts::OS,
                "arch": std::env::consts::ARCH,
            });
            OutputRenderer::render_value(format, &info)?;
            Ok(())
        }
        Commands::Completions { shell } => {
            use clap::CommandFactory;
            let mut cmd = Cli::command();
            clap_complete::generate(*shell, &mut cmd, "mog", &mut std::io::stdout());
            Ok(())
        }
        Commands::Config { command } => commands::config::run(command, format).await,
        Commands::Auth { command } => commands::auth::run(cli, command, format).await,
        Commands::Mail { command } => commands::mail::run(cli, command, format).await,
        Commands::Calendar { command } => commands::calendar::run(cli, command, format).await,
        Commands::Files { command } => commands::files::run(cli, command, format).await,
        Commands::Graph { command } => commands::graph::run(cli, command, format).await,
    }
}
