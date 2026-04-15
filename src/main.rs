mod api_client;
mod auth;
mod commands;
mod models;
mod output;
mod tunnel;

use clap::{Parser, Subcommand};
use models::{AuthType, BastionSku, OutputFormat};

fn parse_key_val(s: &str) -> Result<(String, String), String> {
    let pos = s.find('=').ok_or_else(|| format!("invalid KEY=value: no `=` found in `{s}`"))?;
    Ok((s[..pos].to_string(), s[pos + 1..].to_string()))
}

#[derive(Parser)]
#[command(name = "azcli", version, about = "Azure CLI equivalent with Bastion plugin")]
struct Cli {
    #[arg(short = 'o', long = "output", value_enum, global = true, default_value_t = OutputFormat::Json)]
    output: OutputFormat,

    #[arg(long = "subscription", global = true)]
    subscription: Option<String>,

    #[command(subcommand)]
    command: CliCommand,
}

#[derive(Subcommand)]
enum CliCommand {
    Login {
        #[arg(long)]
        tenant: Option<String>,

        #[arg(long)]
        use_device_code: bool,

        #[arg(long)]
        service_principal: bool,

        #[arg(long)]
        client_id: Option<String>,

        #[arg(long)]
        client_secret: Option<String>,

        #[arg(long)]
        identity: bool,
    },

    Logout,

    Account {
        #[command(subcommand)]
        command: AccountCommand,
    },

    Network {
        #[command(subcommand)]
        command: NetworkCommand,
    },
}

#[derive(Subcommand)]
enum AccountCommand {
    Show,
}

#[derive(Subcommand)]
enum NetworkCommand {
    Bastion {
        #[command(subcommand)]
        command: BastionCommand,
    },
}

#[derive(Subcommand)]
enum BastionCommand {
    Create {
        #[arg(short, long)]
        name: String,
        #[arg(short, long)]
        resource_group: String,
        #[arg(short, long)]
        location: String,
        #[arg(long)]
        vnet_name: String,
        #[arg(long)]
        public_ip_address: Option<String>,
        #[arg(long, value_enum, default_value_t = BastionSku::Standard)]
        sku: BastionSku,
        /// Enable native client tunneling.
        #[arg(long)]
        enable_tunneling: bool,
        /// Enable IP Connect feature.
        #[arg(long)]
        enable_ip_connect: bool,
        /// Enable file copy feature.
        #[arg(long)]
        file_copy: bool,
        /// Disable copy/paste feature.
        #[arg(long)]
        disable_copy_paste: bool,
        /// Enable Kerberos authentication.
        #[arg(long)]
        kerberos: bool,
        /// Enable session recording (Premium SKU only).
        #[arg(long)]
        session_recording: bool,
        /// Enable shareable link.
        #[arg(long)]
        shareable_link: bool,
        /// Network ACL IP rules (Developer SKU only), space-separated CIDRs.
        #[arg(long, value_delimiter = ' ', num_args = 1..)]
        network_acls_ips: Option<Vec<String>>,
        /// Availability zones, space-separated.
        #[arg(long, value_delimiter = ' ', num_args = 1..)]
        zones: Option<Vec<String>>,
        /// Resource tags in key=value format, space-separated.
        #[arg(long, value_parser = parse_key_val, num_args = 1..)]
        tags: Option<Vec<(String, String)>>,
    },
    Delete {
        #[arg(short, long)]
        name: String,
        #[arg(short, long)]
        resource_group: String,
    },
    List {
        #[arg(short, long)]
        resource_group: Option<String>,
    },
    Show {
        #[arg(short, long)]
        name: String,
        #[arg(short, long)]
        resource_group: String,
    },
    Update {
        #[arg(short, long)]
        name: String,
        #[arg(short, long)]
        resource_group: String,
        #[arg(long, value_enum)]
        sku: Option<BastionSku>,
        /// Enable native client tunneling.
        #[arg(long)]
        enable_tunneling: Option<bool>,
        /// Enable IP Connect feature.
        #[arg(long)]
        enable_ip_connect: Option<bool>,
        /// Enable file copy feature.
        #[arg(long)]
        file_copy: Option<bool>,
        /// Disable copy/paste feature.
        #[arg(long)]
        disable_copy_paste: Option<bool>,
        /// Enable Kerberos authentication.
        #[arg(long)]
        kerberos: Option<bool>,
        /// Enable session recording (Premium SKU only).
        #[arg(long)]
        session_recording: Option<bool>,
        /// Enable shareable link.
        #[arg(long)]
        shareable_link: Option<bool>,
        /// Network ACL IP rules (Developer SKU only), space-separated CIDRs.
        #[arg(long, value_delimiter = ' ', num_args = 1..)]
        network_acls_ips: Option<Vec<String>>,
        /// Resource tags in key=value format, space-separated.
        #[arg(long, value_parser = parse_key_val, num_args = 1..)]
        tags: Option<Vec<(String, String)>>,
    },
    Ssh {
        #[arg(short, long)]
        name: String,
        #[arg(short, long)]
        resource_group: String,
        #[arg(long, value_enum)]
        auth_type: AuthType,
        #[arg(long)]
        target_resource_id: Option<String>,
        #[arg(long)]
        target_ip_address: Option<String>,
        #[arg(long, default_value_t = 22)]
        resource_port: u16,
        #[arg(long)]
        username: Option<String>,
        #[arg(long)]
        ssh_key: Option<String>,
        #[arg(last = true)]
        ssh_args: Vec<String>,
    },
    Rdp {
        #[arg(short, long)]
        name: String,
        #[arg(short, long)]
        resource_group: String,
        #[arg(long)]
        target_resource_id: Option<String>,
        #[arg(long)]
        target_ip_address: Option<String>,
        #[arg(long, default_value_t = 3389)]
        resource_port: u16,
        #[arg(long, value_enum)]
        auth_type: Option<AuthType>,
        /// Use tunnel mode instead of web-based RDP gateway.
        #[arg(long)]
        disable_gateway: bool,
        /// Open RDP file in edit mode.
        #[arg(long)]
        configure: bool,
        /// Enable MFA for AAD auth.
        #[arg(long)]
        enable_mfa: bool,
    },
    Tunnel {
        #[arg(short, long)]
        name: String,
        #[arg(short, long)]
        resource_group: String,
        #[arg(long)]
        target_resource_id: Option<String>,
        #[arg(long)]
        target_ip_address: Option<String>,
        #[arg(long)]
        resource_port: u16,
        #[arg(long)]
        port: u16,
        #[arg(long)]
        timeout: Option<u64>,
    },
    Wait {
        #[arg(short, long)]
        name: String,
        #[arg(short, long)]
        resource_group: String,
        /// Wait until created.
        #[arg(long)]
        created: bool,
        /// Wait until updated.
        #[arg(long)]
        updated: bool,
        /// Wait until deleted.
        #[arg(long)]
        deleted: bool,
        /// Wait until provisioning state is 'Succeeded'.
        #[arg(long)]
        exists: bool,
        /// Polling interval in seconds.
        #[arg(long, default_value_t = 30)]
        interval: u64,
        /// Maximum wait time in seconds.
        #[arg(long, default_value_t = 3600)]
        timeout: u64,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn")),
        )
        .init();

    let cli = Cli::parse();

    let output_format = cli.output;
    let subscription = cli.subscription;

    match cli.command {
        CliCommand::Login {
            tenant,
            use_device_code,
            service_principal,
            client_id,
            client_secret,
            identity,
        } => {
            let mut provider = auth::TokenProvider::load(subscription)?;

            if identity {
                provider
                    .login_managed_identity(client_id.as_deref())
                    .await?;
            } else if service_principal {
                let tenant = tenant.as_deref().ok_or_else(|| {
                    anyhow::anyhow!("--tenant is required for service principal login")
                })?;
                let cid = client_id.as_deref().ok_or_else(|| {
                    anyhow::anyhow!("--client-id is required for service principal login")
                })?;
                let secret = client_secret.as_deref().ok_or_else(|| {
                    anyhow::anyhow!("--client-secret is required for service principal login")
                })?;
                provider
                    .login_service_principal(tenant, cid, secret)
                    .await?;
            } else if use_device_code {
                provider.login_device_code(tenant.as_deref()).await?;
            } else {
                provider.login_interactive(tenant.as_deref()).await?;
            }

            Ok(())
        }

        CliCommand::Logout => {
            let mut provider = auth::TokenProvider::load(subscription)?;
            provider.logout()
        }

        CliCommand::Account { command } => match command {
            AccountCommand::Show => {
                let provider = auth::TokenProvider::load(subscription)?;
                provider.show_account()
            }
        },

        CliCommand::Network { command } => match command {
            NetworkCommand::Bastion { command } => {
                handle_bastion(command, output_format, subscription).await?;
                Ok(())
            }
        },
    }
}

async fn handle_bastion(
    cmd: BastionCommand,
    output_format: OutputFormat,
    subscription: Option<String>,
) -> anyhow::Result<()> {
    let mut provider = auth::TokenProvider::load(subscription)?;
    let access_token = provider.get_access_token().await?;
    let subscription_id = provider.get_subscription_id_or_fallback().await?;

    let client = api_client::BastionClient::with_token(access_token, subscription_id);

    match cmd {
        BastionCommand::Create {
            name,
            resource_group,
            location,
            vnet_name,
            public_ip_address,
            sku,
            enable_tunneling,
            enable_ip_connect,
            file_copy,
            disable_copy_paste,
            kerberos,
            session_recording,
            shareable_link,
            network_acls_ips,
            zones,
            tags,
        } => {
            let tags_map = tags.map(|v| v.into_iter().collect());
            let value = commands::create::execute_with_client(
                &client,
                &resource_group,
                &name,
                &location,
                &vnet_name,
                public_ip_address.as_deref(),
                sku,
                enable_tunneling,
                enable_ip_connect,
                file_copy,
                disable_copy_paste,
                kerberos,
                session_recording,
                shareable_link,
                network_acls_ips,
                zones,
                tags_map,
            )
            .await?;
            output::print_output(&value, output_format)
        }
        BastionCommand::Delete {
            name,
            resource_group,
        } => client.delete(&resource_group, &name).await,
        BastionCommand::List { resource_group } => {
            let value = commands::list::execute_with_client(&client, resource_group.as_deref()).await?;
            output::print_output(&value, output_format)
        }
        BastionCommand::Show {
            name,
            resource_group,
        } => {
            let value = commands::show::execute_with_client(&client, &resource_group, &name).await?;
            output::print_output(&value, output_format)
        }
        BastionCommand::Update {
            name,
            resource_group,
            sku,
            enable_tunneling,
            enable_ip_connect,
            file_copy,
            disable_copy_paste,
            kerberos,
            session_recording,
            shareable_link,
            network_acls_ips,
            tags,
        } => {
            let tags_map = tags.map(|v| v.into_iter().collect());
            let value = commands::update::execute_with_client(
                &client,
                &resource_group,
                &name,
                sku,
                enable_tunneling,
                enable_ip_connect,
                file_copy,
                disable_copy_paste,
                kerberos,
                session_recording,
                shareable_link,
                network_acls_ips,
                tags_map,
            )
            .await?;
            output::print_output(&value, output_format)
        }
        BastionCommand::Ssh {
            name,
            resource_group,
            auth_type,
            target_resource_id,
            target_ip_address,
            resource_port,
            username,
            ssh_key,
            ssh_args,
        } => {
            commands::ssh::execute_with_client(
                &client,
                &resource_group,
                &name,
                auth_type,
                target_resource_id.as_deref(),
                target_ip_address.as_deref(),
                resource_port,
                username.as_deref(),
                ssh_key.as_deref(),
                ssh_args,
            )
            .await
        }
        BastionCommand::Rdp {
            name,
            resource_group,
            target_resource_id,
            target_ip_address,
            resource_port,
            auth_type,
            disable_gateway,
            configure,
            enable_mfa,
        } => {
            commands::rdp::execute_with_client(
                &client,
                &resource_group,
                &name,
                target_resource_id.as_deref(),
                target_ip_address.as_deref(),
                resource_port,
                auth_type,
                disable_gateway,
                configure,
                enable_mfa,
            )
            .await
        }
        BastionCommand::Tunnel {
            name,
            resource_group,
            target_resource_id,
            target_ip_address,
            resource_port,
            port,
            timeout,
        } => {
            commands::tunnel::execute_with_client(
                &client,
                &resource_group,
                &name,
                target_resource_id.as_deref(),
                target_ip_address.as_deref(),
                resource_port,
                port,
                timeout,
            )
            .await
        }
        BastionCommand::Wait {
            name,
            resource_group,
            created,
            updated,
            deleted,
            exists,
            interval,
            timeout,
        } => {
            commands::wait::execute_with_client(
                &client,
                &resource_group,
                &name,
                created,
                updated,
                deleted,
                exists,
                interval,
                timeout,
            )
            .await
        }
    }
}
