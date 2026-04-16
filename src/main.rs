mod api_client;
mod arm_client;
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

    Group {
        #[command(subcommand)]
        command: GroupCommand,
    },

    Vm {
        #[command(subcommand)]
        command: VmCommand,
    },

    Vmss {
        #[command(subcommand)]
        command: VmssCommand,
    },

    Deployment {
        #[command(subcommand)]
        command: DeploymentCommand,
    },

    Network {
        #[command(subcommand)]
        command: NetworkCommand,
    },

    Rest {
        #[arg(short = 'u', long = "url", alias = "uri")]
        url: String,

        #[arg(short = 'm', long, default_value = "get")]
        method: String,

        #[arg(short = 'b', long)]
        body: Option<String>,

        #[arg(long, value_parser = parse_key_val, num_args = 1..)]
        headers: Option<Vec<(String, String)>>,

        #[arg(long = "uri-parameters", alias = "url-parameters", value_parser = parse_key_val, num_args = 1..)]
        uri_parameters: Option<Vec<(String, String)>>,

        #[arg(long)]
        resource: Option<String>,

        #[arg(long)]
        skip_authorization_header: bool,

        #[arg(long)]
        output_file: Option<String>,
    },
}

#[derive(Subcommand)]
enum AccountCommand {
    Show,
}

#[derive(Subcommand)]
enum GroupCommand {
    List,
    Show {
        #[arg(short, long)]
        name: String,
    },
}

#[derive(Subcommand)]
enum VmCommand {
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
    Start {
        #[arg(short, long)]
        name: String,
        #[arg(short, long)]
        resource_group: String,
    },
    Stop {
        #[arg(short, long)]
        name: String,
        #[arg(short, long)]
        resource_group: String,
        #[arg(long)]
        no_wait: bool,
    },
    Deallocate {
        #[arg(short, long)]
        name: String,
        #[arg(short, long)]
        resource_group: String,
        #[arg(long)]
        no_wait: bool,
    },
}

#[derive(Subcommand)]
enum VmssCommand {
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
    ListInstances {
        #[arg(short, long)]
        name: String,
        #[arg(short, long)]
        resource_group: String,
        #[arg(long)]
        expand: Option<String>,
    },
    ListSkus {
        #[arg(short, long)]
        name: String,
        #[arg(short, long)]
        resource_group: String,
    },
    ListInstancePublicIps {
        #[arg(short, long)]
        name: String,
        #[arg(short, long)]
        resource_group: String,
    },
    ListInstanceConnectionInfo {
        #[arg(short, long)]
        name: String,
        #[arg(short, long)]
        resource_group: String,
    },
    Scale {
        #[arg(short, long)]
        name: String,
        #[arg(short, long)]
        resource_group: String,
        #[arg(long)]
        new_capacity: i64,
    },
    Start {
        #[arg(short, long)]
        name: String,
        #[arg(short, long)]
        resource_group: String,
        #[arg(long, num_args = 1..)]
        instance_ids: Option<Vec<String>>,
    },
    Stop {
        #[arg(short, long)]
        name: String,
        #[arg(short, long)]
        resource_group: String,
        #[arg(long, num_args = 1..)]
        instance_ids: Option<Vec<String>>,
    },
    UpdateInstances {
        #[arg(short, long)]
        name: String,
        #[arg(short, long)]
        resource_group: String,
        #[arg(long, num_args = 1.., required = true)]
        instance_ids: Vec<String>,
    },
    Wait {
        #[arg(short, long)]
        name: String,
        #[arg(short, long)]
        resource_group: String,
        #[arg(long)]
        created: bool,
        #[arg(long)]
        updated: bool,
        #[arg(long)]
        deleted: bool,
        #[arg(long)]
        exists: bool,
        #[arg(long, default_value_t = 30)]
        interval: u64,
        #[arg(long, default_value_t = 3600)]
        timeout: u64,
    },
}

#[derive(Subcommand)]
enum DeploymentCommand {
    Group {
        #[command(subcommand)]
        command: DeploymentGroupCommand,
    },
    Operation {
        #[command(subcommand)]
        command: DeploymentOperationCommand,
    },
}

#[derive(Subcommand)]
enum DeploymentGroupCommand {
    List {
        #[arg(short, long)]
        resource_group: String,
    },
    Show {
        #[arg(short, long)]
        name: String,
        #[arg(short, long)]
        resource_group: String,
    },
    Export {
        #[arg(short, long)]
        name: String,
        #[arg(short, long)]
        resource_group: String,
    },
    Validate {
        #[arg(short, long)]
        resource_group: String,
        #[arg(short, long)]
        name: Option<String>,
        #[arg(short = 'f', long)]
        template_file: Option<String>,
        #[arg(short = 'u', long)]
        template_uri: Option<String>,
        #[arg(short, long)]
        parameters: Option<String>,
        #[arg(long, default_value = "Incremental")]
        mode: String,
    },
    WhatIf {
        #[arg(short, long)]
        resource_group: String,
        #[arg(short, long)]
        name: Option<String>,
        #[arg(short = 'f', long)]
        template_file: Option<String>,
        #[arg(short = 'u', long)]
        template_uri: Option<String>,
        #[arg(short, long)]
        parameters: Option<String>,
        #[arg(long, default_value = "Incremental")]
        mode: String,
        #[arg(long, default_value = "FullResourcePayloads")]
        result_format: String,
    },
    Cancel {
        #[arg(short, long)]
        name: String,
        #[arg(short, long)]
        resource_group: String,
    },
    Wait {
        #[arg(short, long)]
        name: String,
        #[arg(short, long)]
        resource_group: String,
        #[arg(long)]
        created: bool,
        #[arg(long)]
        updated: bool,
        #[arg(long)]
        deleted: bool,
        #[arg(long)]
        exists: bool,
        #[arg(long, default_value_t = 30)]
        interval: u64,
        #[arg(long, default_value_t = 3600)]
        timeout: u64,
    },
}

#[derive(Subcommand)]
enum DeploymentOperationCommand {
    Group {
        #[command(subcommand)]
        command: DeploymentOperationGroupCommand,
    },
}

#[derive(Subcommand)]
enum DeploymentOperationGroupCommand {
    List {
        #[arg(short, long)]
        name: String,
        #[arg(short, long)]
        resource_group: String,
    },
    Show {
        #[arg(short, long)]
        name: String,
        #[arg(short, long)]
        resource_group: String,
        #[arg(long)]
        operation_id: String,
    },
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

        CliCommand::Group { command } => {
            handle_group(command, output_format, subscription).await
        }

        CliCommand::Vm { command } => {
            handle_vm(command, output_format, subscription).await
        }

        CliCommand::Vmss { command } => {
            handle_vmss(command, output_format, subscription).await
        }

        CliCommand::Deployment { command } => {
            handle_deployment(command, output_format, subscription).await
        }

        CliCommand::Network { command } => match command {
            NetworkCommand::Bastion { command } => {
                handle_bastion(command, output_format, subscription).await?;
                Ok(())
            }
        },

        CliCommand::Rest {
            url,
            method,
            body,
            headers,
            uri_parameters,
            resource: _,
            skip_authorization_header,
            output_file,
        } => {
            let (access_token, subscription_id) = if skip_authorization_header {
                (None, subscription)
            } else {
                let mut provider = auth::TokenProvider::load(subscription)?;
                let token = provider.get_access_token().await?;
                let sub = provider.get_subscription_id_or_fallback().await.ok();
                (Some(token), sub)
            };

            let headers_vec = headers.unwrap_or_default();
            let params_vec = uri_parameters.unwrap_or_default();

            let value = commands::rest::execute(
                access_token.as_deref(),
                &url,
                &method,
                body.as_deref(),
                if headers_vec.is_empty() { None } else { Some(&headers_vec) },
                if params_vec.is_empty() { None } else { Some(&params_vec) },
                skip_authorization_header,
                subscription_id.as_deref(),
            ).await?;

            if let Some(path) = output_file {
                let content = serde_json::to_string_pretty(&value)?;
                std::fs::write(&path, &content)?;
                eprintln!("Response saved to {path}");
                Ok(())
            } else {
                output::print_output(&value, output_format)
            }
        }
    }
}

async fn handle_group(
    cmd: GroupCommand,
    output_format: OutputFormat,
    subscription: Option<String>,
) -> anyhow::Result<()> {
    let mut provider = auth::TokenProvider::load(subscription)?;
    let access_token = provider.get_access_token().await?;
    let subscription_id = provider.get_subscription_id_or_fallback().await?;

    let client = arm_client::ArmClient::new(access_token, subscription_id);

    match cmd {
        GroupCommand::List => {
            let value = commands::group::list::execute(&client).await?;
            output::print_output(&value, output_format)
        }
        GroupCommand::Show { name } => {
            let value = commands::group::show::execute(&client, &name).await?;
            output::print_output(&value, output_format)
        }
    }
}

async fn handle_vm(
    cmd: VmCommand,
    output_format: OutputFormat,
    subscription: Option<String>,
) -> anyhow::Result<()> {
    let mut provider = auth::TokenProvider::load(subscription)?;
    let access_token = provider.get_access_token().await?;
    let subscription_id = provider.get_subscription_id_or_fallback().await?;

    let client = arm_client::ArmClient::new(access_token, subscription_id);

    match cmd {
        VmCommand::List { resource_group } => {
            let value = commands::vm::list::execute(&client, resource_group.as_deref()).await?;
            output::print_output(&value, output_format)
        }
        VmCommand::Show { name, resource_group } => {
            let value = commands::vm::show::execute(&client, &resource_group, &name).await?;
            output::print_output(&value, output_format)
        }
        VmCommand::Start { name, resource_group } => {
            commands::vm::start::execute(&client, &resource_group, &name).await
        }
        VmCommand::Stop { name, resource_group, no_wait } => {
            commands::vm::stop::execute(&client, &resource_group, &name, no_wait).await
        }
        VmCommand::Deallocate { name, resource_group, no_wait } => {
            commands::vm::deallocate::execute(&client, &resource_group, &name, no_wait).await
        }
    }
}

async fn handle_vmss(
    cmd: VmssCommand,
    output_format: OutputFormat,
    subscription: Option<String>,
) -> anyhow::Result<()> {
    let mut provider = auth::TokenProvider::load(subscription)?;
    let access_token = provider.get_access_token().await?;
    let subscription_id = provider.get_subscription_id_or_fallback().await?;

    let client = arm_client::ArmClient::new(access_token, subscription_id);

    match cmd {
        VmssCommand::List { resource_group } => {
            let value = commands::vmss::list::execute(&client, resource_group.as_deref()).await?;
            output::print_output(&value, output_format)
        }
        VmssCommand::Show { name, resource_group } => {
            let value = commands::vmss::show::execute(&client, &resource_group, &name).await?;
            output::print_output(&value, output_format)
        }
        VmssCommand::ListInstances { name, resource_group, expand } => {
            let value = commands::vmss::list_instances::execute(&client, &resource_group, &name, expand.as_deref()).await?;
            output::print_output(&value, output_format)
        }
        VmssCommand::ListSkus { name, resource_group } => {
            let value = commands::vmss::list_skus::execute(&client, &resource_group, &name).await?;
            output::print_output(&value, output_format)
        }
        VmssCommand::ListInstancePublicIps { name, resource_group } => {
            let value = commands::vmss::list_instance_public_ips::execute(&client, &resource_group, &name).await?;
            output::print_output(&value, output_format)
        }
        VmssCommand::ListInstanceConnectionInfo { name, resource_group } => {
            let value = commands::vmss::list_instance_connection_info::execute(&client, &resource_group, &name).await?;
            output::print_output(&value, output_format)
        }
        VmssCommand::Scale { name, resource_group, new_capacity } => {
            commands::vmss::scale::execute(&client, &resource_group, &name, new_capacity).await
        }
        VmssCommand::Start { name, resource_group, instance_ids } => {
            commands::vmss::start::execute(&client, &resource_group, &name, instance_ids.as_deref()).await
        }
        VmssCommand::Stop { name, resource_group, instance_ids } => {
            commands::vmss::stop::execute(&client, &resource_group, &name, instance_ids.as_deref()).await
        }
        VmssCommand::UpdateInstances { name, resource_group, instance_ids } => {
            commands::vmss::update_instances::execute(&client, &resource_group, &name, &instance_ids).await
        }
        VmssCommand::Wait { name, resource_group, created, updated, deleted, exists, interval, timeout } => {
            commands::vmss::wait::execute(&client, &resource_group, &name, created, updated, deleted, exists, interval, timeout).await
        }
    }
}

async fn handle_deployment(
    cmd: DeploymentCommand,
    output_format: OutputFormat,
    subscription: Option<String>,
) -> anyhow::Result<()> {
    let mut provider = auth::TokenProvider::load(subscription)?;
    let access_token = provider.get_access_token().await?;
    let subscription_id = provider.get_subscription_id_or_fallback().await?;

    let client = arm_client::ArmClient::new(access_token, subscription_id);

    match cmd {
        DeploymentCommand::Group { command } => match command {
            DeploymentGroupCommand::List { resource_group } => {
                let value = commands::deployment::group::list::execute(&client, &resource_group).await?;
                output::print_output(&value, output_format)
            }
            DeploymentGroupCommand::Show { name, resource_group } => {
                let value = commands::deployment::group::show::execute(&client, &resource_group, &name).await?;
                output::print_output(&value, output_format)
            }
            DeploymentGroupCommand::Export { name, resource_group } => {
                let value = commands::deployment::group::export::execute(&client, &resource_group, &name).await?;
                output::print_output(&value, output_format)
            }
            DeploymentGroupCommand::Validate { resource_group, name, template_file, template_uri, parameters, mode } => {
                let deploy_name = name.unwrap_or_else(|| "validation".to_string());
                let value = commands::deployment::group::validate::execute(
                    &client, &resource_group, &deploy_name,
                    template_file.as_deref(), template_uri.as_deref(),
                    parameters.as_deref(), &mode,
                ).await?;
                output::print_output(&value, output_format)
            }
            DeploymentGroupCommand::WhatIf { resource_group, name, template_file, template_uri, parameters, mode, result_format } => {
                let deploy_name = name.unwrap_or_else(|| "what-if".to_string());
                let value = commands::deployment::group::what_if::execute(
                    &client, &resource_group, &deploy_name,
                    template_file.as_deref(), template_uri.as_deref(),
                    parameters.as_deref(), &mode, Some(&result_format),
                ).await?;
                output::print_output(&value, output_format)
            }
            DeploymentGroupCommand::Cancel { name, resource_group } => {
                commands::deployment::group::cancel::execute(&client, &resource_group, &name).await
            }
            DeploymentGroupCommand::Wait { name, resource_group, created, updated, deleted, exists, interval, timeout } => {
                commands::deployment::group::wait::execute(&client, &resource_group, &name, created, updated, deleted, exists, interval, timeout).await
            }
        },
        DeploymentCommand::Operation { command } => match command {
            DeploymentOperationCommand::Group { command } => match command {
                DeploymentOperationGroupCommand::List { name, resource_group } => {
                    let value = commands::deployment::operation::group::list::execute(&client, &resource_group, &name).await?;
                    output::print_output(&value, output_format)
                }
                DeploymentOperationGroupCommand::Show { name, resource_group, operation_id } => {
                    let value = commands::deployment::operation::group::show::execute(&client, &resource_group, &name, &operation_id).await?;
                    output::print_output(&value, output_format)
                }
            },
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
