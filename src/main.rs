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

    #[arg(long = "query", global = true)]
    query: Option<String>,

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

    Role {
        #[command(subcommand)]
        command: RoleCommand,
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

    Disk {
        #[command(subcommand)]
        command: DiskCommand,
    },

    Image {
        #[command(subcommand)]
        command: ImageCommand,
    },

    Sig {
        #[command(subcommand)]
        command: SigCommand,
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
    Show {
        #[arg(short = 'n', long)]
        name: Option<String>,
    },
    List,
    Set {
        #[arg(short = 'n', long)]
        name: String,
    },
    ListLocations {
        #[arg(long)]
        subscription_id: Option<String>,
    },
    GetAccessToken,
    Clear,
}

#[derive(Subcommand)]
enum RoleCommand {
    Assignment {
        #[command(subcommand)]
        command: RoleAssignmentCommand,
    },
    Definition {
        #[command(subcommand)]
        command: RoleDefinitionCommand,
    },
    Pim {
        #[command(subcommand)]
        command: RolePimCommand,
    },
}

#[derive(Subcommand)]
enum RoleAssignmentCommand {
    List {
        #[arg(long)]
        assignee: Option<String>,
        #[arg(long)]
        role: Option<String>,
        #[arg(long)]
        scope: Option<String>,
        #[arg(short = 'g', long)]
        resource_group: Option<String>,
        #[arg(long)]
        include_groups: bool,
        #[arg(long)]
        all: bool,
    },
    Show {
        #[arg(long)]
        ids: Option<String>,
        #[arg(short = 'n', long)]
        name: Option<String>,
        #[arg(long)]
        scope: Option<String>,
    },
}

#[derive(Subcommand)]
enum RoleDefinitionCommand {
    List {
        #[arg(short = 'n', long)]
        name: Option<String>,
        #[arg(long)]
        scope: Option<String>,
        #[arg(long)]
        custom_role_only: bool,
    },
    Show {
        #[arg(short = 'n', long)]
        name: String,
        #[arg(long)]
        scope: Option<String>,
    },
}

#[derive(Subcommand)]
enum RolePimCommand {
    List {
        #[arg(long)]
        scope: Option<String>,
    },
    Status {
        #[arg(long)]
        scope: Option<String>,
    },
    Activate {
        #[arg(short = 'r', long)]
        role: String,
        #[arg(short = 'j', long, default_value = "Activated via azcli")]
        justification: String,
        #[arg(short = 'd', long, default_value = "PT8H")]
        duration: String,
        #[arg(long)]
        scope: Option<String>,
    },
    Deactivate {
        #[arg(short = 'r', long)]
        role: String,
        #[arg(long)]
        scope: Option<String>,
    },
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
        #[arg(short = 'g', long)]
        resource_group: Option<String>,
    },
    Show {
        #[arg(short, long)]
        name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
    },
    Start {
        #[arg(short, long)]
        name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
    },
    Stop {
        #[arg(short, long)]
        name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
        #[arg(long)]
        no_wait: bool,
    },
    Deallocate {
        #[arg(short, long)]
        name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
        #[arg(long)]
        no_wait: bool,
    },
    #[command(name = "get-instance-view")]
    GetInstanceView {
        #[arg(short, long)]
        name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
    },
    #[command(name = "list-ip-addresses")]
    ListIpAddresses {
        #[arg(short, long)]
        name: Option<String>,
        #[arg(short = 'g', long)]
        resource_group: Option<String>,
    },
    #[command(name = "list-sizes")]
    ListSizes {
        #[arg(short, long)]
        location: String,
    },
    #[command(name = "list-skus")]
    ListSkus {
        #[arg(short, long)]
        location: Option<String>,
        #[arg(short = 't', long)]
        resource_type: Option<String>,
        #[arg(short, long)]
        size: Option<String>,
        #[arg(short, long)]
        zone: bool,
    },
    #[command(name = "list-usage")]
    ListUsage {
        #[arg(short, long)]
        location: String,
    },
    #[command(name = "list-vm-resize-options")]
    ListVmResizeOptions {
        #[arg(short, long)]
        name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
    },
    Restart {
        #[arg(short, long)]
        name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
        #[arg(long)]
        no_wait: bool,
    },
    Create {
        #[arg(short, long)]
        name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
        #[arg(long)]
        image: String,
        #[arg(long, default_value = "Standard_DS1_v2")]
        size: String,
        #[arg(short, long)]
        location: Option<String>,
        #[arg(long)]
        admin_username: Option<String>,
        #[arg(long)]
        admin_password: Option<String>,
        #[arg(long)]
        ssh_key_value: Option<String>,
        #[arg(long)]
        authentication_type: Option<String>,
        #[arg(long)]
        subnet: Option<String>,
        #[arg(long)]
        os_disk_size_gb: Option<i64>,
        #[arg(long, num_args = 1..)]
        data_disk_sizes_gb: Vec<i64>,
        #[arg(long, num_args = 1..)]
        tags: Vec<String>,
        #[arg(long)]
        no_wait: bool,
    },
    Delete {
        #[arg(short, long)]
        name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
        #[arg(long)]
        force_deletion: bool,
        #[arg(long)]
        no_wait: bool,
    },
    Update {
        #[arg(short, long)]
        name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
        #[arg(long, num_args = 1..)]
        set: Vec<String>,
        #[arg(long)]
        no_wait: bool,
    },
    Resize {
        #[arg(short, long)]
        name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
        #[arg(long)]
        size: String,
        #[arg(long)]
        no_wait: bool,
    },
    Redeploy {
        #[arg(short, long)]
        name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
        #[arg(long)]
        no_wait: bool,
    },
    Reimage {
        #[arg(short, long)]
        name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
        #[arg(long)]
        no_wait: bool,
    },
    Reapply {
        #[arg(short, long)]
        name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
        #[arg(long)]
        no_wait: bool,
    },
    #[command(name = "perform-maintenance")]
    PerformMaintenance {
        #[arg(short, long)]
        name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
    },
    #[command(name = "simulate-eviction")]
    SimulateEviction {
        #[arg(short, long)]
        name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
    },
    Generalize {
        #[arg(short, long)]
        name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
    },
    Capture {
        #[arg(short, long)]
        name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
        #[arg(long)]
        vhd_name_prefix: String,
        #[arg(long, default_value = "vhds")]
        storage_container: String,
        #[arg(long)]
        overwrite: bool,
    },
    Convert {
        #[arg(short, long)]
        name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
    },
    #[command(name = "assess-patches")]
    AssessPatches {
        #[arg(short, long)]
        name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
    },
    #[command(name = "install-patches")]
    InstallPatches {
        #[arg(short, long)]
        name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
        #[arg(long)]
        maximum_duration: String,
        #[arg(long)]
        reboot_setting: String,
        #[arg(long, num_args = 1..)]
        classifications_to_include_linux: Vec<String>,
        #[arg(long, num_args = 1..)]
        classifications_to_include_win: Vec<String>,
    },
    #[command(name = "auto-shutdown")]
    AutoShutdown {
        #[arg(short, long)]
        name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
        #[arg(long)]
        time: Option<String>,
        #[arg(long)]
        off: bool,
        #[arg(long)]
        email: Option<String>,
        #[arg(long)]
        webhook: Option<String>,
        #[arg(short, long)]
        location: Option<String>,
    },
    #[command(name = "open-port")]
    OpenPort {
        #[arg(short, long)]
        name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
        #[arg(long)]
        port: String,
        #[arg(long, default_value = "900")]
        priority: i64,
        #[arg(long)]
        nsg_name: Option<String>,
        #[arg(long)]
        apply_to_subnet: bool,
    },
    Disk {
        #[command(subcommand)]
        command: VmDiskCommand,
    },
    Nic {
        #[command(subcommand)]
        command: VmNicCommand,
    },
    #[command(name = "run-command")]
    RunCommand {
        #[command(subcommand)]
        command: VmRunCommandCommand,
    },
    Wait {
        #[arg(short, long)]
        name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
        #[arg(long)]
        created: bool,
        #[arg(long)]
        updated: bool,
        #[arg(long)]
        deleted: bool,
        #[arg(long)]
        exists: bool,
        #[arg(long, default_value = "30")]
        interval: u64,
        #[arg(long, default_value = "3600")]
        timeout: u64,
    },
}

#[derive(Subcommand)]
enum VmDiskCommand {
    Attach {
        #[arg(long = "vm-name")]
        vm_name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
        #[arg(short, long)]
        name: String,
        #[arg(long)]
        new: bool,
        #[arg(long = "size-gb", short = 'z')]
        size_gb: Option<i64>,
        #[arg(long)]
        sku: Option<String>,
        #[arg(long)]
        lun: Option<i64>,
        #[arg(long)]
        caching: Option<String>,
        #[arg(long = "enable-write-accelerator")]
        enable_write_accelerator: bool,
    },
    Detach {
        #[arg(long = "vm-name")]
        vm_name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
        #[arg(short, long)]
        name: String,
        #[arg(long = "force-detach")]
        force_detach: bool,
    },
}

#[derive(Subcommand)]
enum VmNicCommand {
    List {
        #[arg(long = "vm-name")]
        vm_name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
    },
    Show {
        #[arg(long = "vm-name")]
        vm_name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
        #[arg(long)]
        nic: String,
    },
    Add {
        #[arg(long = "vm-name")]
        vm_name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
        #[arg(long, num_args = 1.., required = true)]
        nics: Vec<String>,
        #[arg(long = "primary-nic")]
        primary_nic: Option<String>,
    },
    Remove {
        #[arg(long = "vm-name")]
        vm_name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
        #[arg(long, num_args = 1.., required = true)]
        nics: Vec<String>,
        #[arg(long = "primary-nic")]
        primary_nic: Option<String>,
    },
    Set {
        #[arg(long = "vm-name")]
        vm_name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
        #[arg(long, num_args = 1.., required = true)]
        nics: Vec<String>,
        #[arg(long = "primary-nic")]
        primary_nic: Option<String>,
    },
}

#[derive(Subcommand)]
enum VmRunCommandCommand {
    Invoke {
        #[arg(long = "vm-name")]
        vm_name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
        #[arg(long = "command-id")]
        command_id: String,
        #[arg(long, num_args = 0..)]
        scripts: Vec<String>,
        #[arg(long, num_args = 0..)]
        parameters: Vec<String>,
    },
    List {
        #[arg(long = "vm-name")]
        vm_name: Option<String>,
        #[arg(short = 'g', long)]
        resource_group: Option<String>,
        #[arg(short, long)]
        location: Option<String>,
        #[arg(long = "expand-instance-view")]
        expand_instance_view: bool,
    },
    Show {
        #[arg(long = "vm-name")]
        vm_name: Option<String>,
        #[arg(short = 'g', long)]
        resource_group: Option<String>,
        #[arg(short, long)]
        name: Option<String>,
        #[arg(short, long)]
        location: Option<String>,
        #[arg(long = "command-id")]
        command_id: Option<String>,
        #[arg(long = "instance-view")]
        instance_view: bool,
    },
    Create {
        #[arg(long = "vm-name")]
        vm_name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
        #[arg(short, long)]
        name: String,
        #[arg(short, long)]
        location: Option<String>,
        #[arg(long)]
        script: Option<String>,
        #[arg(long = "script-uri")]
        script_uri: Option<String>,
        #[arg(long = "command-id")]
        command_id: Option<String>,
        #[arg(long, num_args = 0..)]
        parameters: Vec<String>,
        #[arg(long = "protected-parameters", num_args = 0..)]
        protected_parameters: Vec<String>,
        #[arg(long = "run-as-user")]
        run_as_user: Option<String>,
        #[arg(long = "run-as-password")]
        run_as_password: Option<String>,
        #[arg(long = "async-execution")]
        async_execution: bool,
        #[arg(long = "timeout-in-seconds")]
        timeout_in_seconds: Option<i64>,
        #[arg(long = "output-blob-uri")]
        output_blob_uri: Option<String>,
        #[arg(long = "error-blob-uri")]
        error_blob_uri: Option<String>,
    },
    Update {
        #[arg(long = "vm-name")]
        vm_name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
        #[arg(short, long)]
        name: String,
        #[arg(long)]
        script: Option<String>,
        #[arg(long = "script-uri")]
        script_uri: Option<String>,
        #[arg(long = "command-id")]
        command_id: Option<String>,
        #[arg(long, num_args = 0..)]
        parameters: Vec<String>,
        #[arg(long = "protected-parameters", num_args = 0..)]
        protected_parameters: Vec<String>,
        #[arg(long = "run-as-user")]
        run_as_user: Option<String>,
        #[arg(long = "run-as-password")]
        run_as_password: Option<String>,
        #[arg(long = "timeout-in-seconds")]
        timeout_in_seconds: Option<i64>,
        #[arg(long = "output-blob-uri")]
        output_blob_uri: Option<String>,
        #[arg(long = "error-blob-uri")]
        error_blob_uri: Option<String>,
    },
    Delete {
        #[arg(long = "vm-name")]
        vm_name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
        #[arg(short, long)]
        name: String,
    },
}

#[derive(Subcommand)]
enum DiskCommand {
    List {
        #[arg(short = 'g', long)]
        resource_group: Option<String>,
    },
    Show {
        #[arg(short, long)]
        name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
    },
    #[command(name = "list-skus")]
    ListSkus {
        #[arg(short, long)]
        location: Option<String>,
        #[arg(short, long)]
        zone: bool,
    },
    Create {
        #[arg(short, long)]
        name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
        #[arg(short, long)]
        location: Option<String>,
        #[arg(long = "size-gb", short = 'z')]
        size_gb: Option<i64>,
        #[arg(long)]
        sku: Option<String>,
        #[arg(long)]
        source: Option<String>,
        #[arg(long)]
        zone: Option<String>,
        #[arg(long = "hyper-v-generation")]
        hyper_v_generation: Option<String>,
        #[arg(long = "os-type")]
        os_type: Option<String>,
    },
    Delete {
        #[arg(short, long)]
        name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
    },
    Update {
        #[arg(short, long)]
        name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
        #[arg(long = "size-gb", short = 'z')]
        size_gb: Option<i64>,
        #[arg(long)]
        sku: Option<String>,
    },
    #[command(name = "grant-access")]
    GrantAccess {
        #[arg(short, long)]
        name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
        #[arg(long = "access-level", default_value = "Read")]
        access_level: String,
        #[arg(long = "duration-in-seconds", default_value = "3600")]
        duration_in_seconds: i64,
    },
    #[command(name = "revoke-access")]
    RevokeAccess {
        #[arg(short, long)]
        name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
    },
}

#[derive(Subcommand)]
enum ImageCommand {
    List {
        #[arg(short = 'g', long)]
        resource_group: Option<String>,
    },
    Show {
        #[arg(short, long)]
        name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
    },
    Builder {
        #[command(subcommand)]
        command: ImageBuilderCommand,
    },
}

#[derive(Subcommand)]
enum ImageBuilderCommand {
    List {
        #[arg(short = 'g', long)]
        resource_group: Option<String>,
    },
    Show {
        #[arg(short, long)]
        name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
    },
    #[command(name = "show-runs")]
    ShowRuns {
        #[arg(short, long)]
        name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
        #[arg(long = "output-name")]
        output_name: Option<String>,
    },
}

#[derive(Subcommand)]
enum SigCommand {
    List {
        #[arg(short = 'g', long)]
        resource_group: Option<String>,
    },
    Show {
        #[arg(short = 'r', long = "gallery-name")]
        gallery_name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
    },
    #[command(name = "list-shared")]
    ListShared {
        #[arg(short, long)]
        location: String,
        #[arg(long = "shared-to")]
        shared_to: Option<String>,
    },
    #[command(name = "show-shared")]
    ShowShared {
        #[arg(short, long)]
        location: String,
        #[arg(short = 'r', long = "gallery-unique-name")]
        gallery_unique_name: String,
    },
    #[command(name = "list-community")]
    ListCommunity {
        #[arg(short, long)]
        location: Option<String>,
    },
    #[command(name = "show-community")]
    ShowCommunity {
        #[arg(short, long)]
        location: String,
        #[arg(short = 'p', long = "public-gallery-name")]
        public_gallery_name: String,
    },
    #[command(name = "image-definition")]
    ImageDefinition {
        #[command(subcommand)]
        command: SigImageDefinitionCommand,
    },
    #[command(name = "image-version")]
    ImageVersion {
        #[command(subcommand)]
        command: SigImageVersionCommand,
    },
}

#[derive(Subcommand)]
enum SigImageDefinitionCommand {
    List {
        #[arg(short = 'g', long)]
        resource_group: String,
        #[arg(short = 'r', long = "gallery-name")]
        gallery_name: String,
    },
    Show {
        #[arg(short = 'g', long)]
        resource_group: String,
        #[arg(short = 'r', long = "gallery-name")]
        gallery_name: String,
        #[arg(short = 'i', long = "gallery-image-definition")]
        gallery_image_definition: String,
    },
    #[command(name = "list-shared")]
    ListShared {
        #[arg(short, long)]
        location: String,
        #[arg(short = 'r', long = "gallery-unique-name")]
        gallery_unique_name: String,
    },
    #[command(name = "show-shared")]
    ShowShared {
        #[arg(short, long)]
        location: String,
        #[arg(short = 'r', long = "gallery-unique-name")]
        gallery_unique_name: String,
        #[arg(short = 'i', long = "gallery-image-definition")]
        gallery_image_definition: String,
    },
    #[command(name = "list-community")]
    ListCommunity {
        #[arg(short, long)]
        location: String,
        #[arg(short = 'p', long = "public-gallery-name")]
        public_gallery_name: String,
    },
    #[command(name = "show-community")]
    ShowCommunity {
        #[arg(short, long)]
        location: String,
        #[arg(short = 'p', long = "public-gallery-name")]
        public_gallery_name: String,
        #[arg(short = 'i', long = "gallery-image-definition")]
        gallery_image_definition: String,
    },
}

#[derive(Subcommand)]
enum SigImageVersionCommand {
    List {
        #[arg(short = 'g', long)]
        resource_group: String,
        #[arg(short = 'r', long = "gallery-name")]
        gallery_name: String,
        #[arg(short = 'i', long = "gallery-image-definition")]
        gallery_image_definition: String,
    },
    Show {
        #[arg(short = 'g', long)]
        resource_group: String,
        #[arg(short = 'r', long = "gallery-name")]
        gallery_name: String,
        #[arg(short = 'i', long = "gallery-image-definition")]
        gallery_image_definition: String,
        #[arg(short = 'e', long = "gallery-image-version")]
        gallery_image_version: String,
    },
    #[command(name = "list-shared")]
    ListShared {
        #[arg(short, long)]
        location: String,
        #[arg(short = 'r', long = "gallery-unique-name")]
        gallery_unique_name: String,
        #[arg(short = 'i', long = "gallery-image-definition")]
        gallery_image_definition: String,
    },
    #[command(name = "show-shared")]
    ShowShared {
        #[arg(short, long)]
        location: String,
        #[arg(short = 'r', long = "gallery-unique-name")]
        gallery_unique_name: String,
        #[arg(short = 'i', long = "gallery-image-definition")]
        gallery_image_definition: String,
        #[arg(short = 'e', long = "gallery-image-version")]
        gallery_image_version: String,
    },
    #[command(name = "list-community")]
    ListCommunity {
        #[arg(short, long)]
        location: String,
        #[arg(short = 'p', long = "public-gallery-name")]
        public_gallery_name: String,
        #[arg(short = 'i', long = "gallery-image-definition")]
        gallery_image_definition: String,
    },
    #[command(name = "show-community")]
    ShowCommunity {
        #[arg(short, long)]
        location: String,
        #[arg(short = 'p', long = "public-gallery-name")]
        public_gallery_name: String,
        #[arg(short = 'i', long = "gallery-image-definition")]
        gallery_image_definition: String,
        #[arg(short = 'e', long = "gallery-image-version")]
        gallery_image_version: String,
    },
}

#[derive(Subcommand)]
enum VmssCommand {
    List {
        #[arg(short = 'g', long)]
        resource_group: Option<String>,
    },
    Show {
        #[arg(short, long)]
        name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
    },
    ListInstances {
        #[arg(short, long)]
        name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
        #[arg(long)]
        expand: Option<String>,
    },
    ListSkus {
        #[arg(short, long)]
        name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
    },
    ListInstancePublicIps {
        #[arg(short, long)]
        name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
    },
    ListInstanceConnectionInfo {
        #[arg(short, long)]
        name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
    },
    Scale {
        #[arg(short, long)]
        name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
        #[arg(long)]
        new_capacity: i64,
    },
    Start {
        #[arg(short, long)]
        name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
        #[arg(long, num_args = 1..)]
        instance_ids: Option<Vec<String>>,
    },
    Stop {
        #[arg(short, long)]
        name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
        #[arg(long, num_args = 1..)]
        instance_ids: Option<Vec<String>>,
    },
    UpdateInstances {
        #[arg(short, long)]
        name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
        #[arg(long, num_args = 1.., required = true)]
        instance_ids: Vec<String>,
    },
    Wait {
        #[arg(short, long)]
        name: String,
        #[arg(short = 'g', long)]
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
    Sub {
        #[command(subcommand)]
        command: DeploymentSubCommand,
    },
    Mg {
        #[command(subcommand)]
        command: DeploymentMgCommand,
    },
    Tenant {
        #[command(subcommand)]
        command: DeploymentTenantCommand,
    },
    Operation {
        #[command(subcommand)]
        command: DeploymentOperationCommand,
    },
}

#[derive(Subcommand)]
enum DeploymentGroupCommand {
    List {
        #[arg(short = 'g', long)]
        resource_group: String,
    },
    Show {
        #[arg(short, long)]
        name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
    },
    Export {
        #[arg(short, long)]
        name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
    },
    Create {
        #[arg(short = 'g', long)]
        resource_group: String,
        #[arg(short, long)]
        name: String,
        #[arg(short = 'f', long)]
        template_file: Option<String>,
        #[arg(short = 'u', long)]
        template_uri: Option<String>,
        #[arg(short, long)]
        parameters: Option<String>,
        #[arg(long, default_value = "Incremental")]
        mode: String,
    },
    Delete {
        #[arg(short, long)]
        name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
    },
    Validate {
        #[arg(short = 'g', long)]
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
        #[arg(short = 'g', long)]
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
        #[arg(short = 'g', long)]
        resource_group: String,
    },
    Wait {
        #[arg(short, long)]
        name: String,
        #[arg(short = 'g', long)]
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
enum DeploymentSubCommand {
    List,
    Show {
        #[arg(short, long)]
        name: String,
    },
    Export {
        #[arg(short, long)]
        name: String,
    },
    Create {
        #[arg(short, long)]
        name: String,
        #[arg(short, long)]
        location: String,
        #[arg(short = 'f', long)]
        template_file: Option<String>,
        #[arg(short = 'u', long)]
        template_uri: Option<String>,
        #[arg(short, long)]
        parameters: Option<String>,
    },
    Delete {
        #[arg(short, long)]
        name: String,
    },
    Validate {
        #[arg(short, long)]
        name: Option<String>,
        #[arg(short, long)]
        location: String,
        #[arg(short = 'f', long)]
        template_file: Option<String>,
        #[arg(short = 'u', long)]
        template_uri: Option<String>,
        #[arg(short, long)]
        parameters: Option<String>,
    },
    WhatIf {
        #[arg(short, long)]
        name: Option<String>,
        #[arg(short, long)]
        location: String,
        #[arg(short = 'f', long)]
        template_file: Option<String>,
        #[arg(short = 'u', long)]
        template_uri: Option<String>,
        #[arg(short, long)]
        parameters: Option<String>,
        #[arg(long, default_value = "FullResourcePayloads")]
        result_format: String,
    },
    Cancel {
        #[arg(short, long)]
        name: String,
    },
    Wait {
        #[arg(short, long)]
        name: String,
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
enum DeploymentMgCommand {
    List {
        #[arg(short, long)]
        management_group_id: String,
    },
    Show {
        #[arg(short, long)]
        name: String,
        #[arg(short, long)]
        management_group_id: String,
    },
    Export {
        #[arg(short, long)]
        name: String,
        #[arg(short, long)]
        management_group_id: String,
    },
    Create {
        #[arg(short, long)]
        name: String,
        #[arg(short, long)]
        management_group_id: String,
        #[arg(short, long)]
        location: String,
        #[arg(short = 'f', long)]
        template_file: Option<String>,
        #[arg(short = 'u', long)]
        template_uri: Option<String>,
        #[arg(short, long)]
        parameters: Option<String>,
    },
    Delete {
        #[arg(short, long)]
        name: String,
        #[arg(short, long)]
        management_group_id: String,
    },
    Validate {
        #[arg(short, long)]
        name: Option<String>,
        #[arg(short, long)]
        management_group_id: String,
        #[arg(short, long)]
        location: String,
        #[arg(short = 'f', long)]
        template_file: Option<String>,
        #[arg(short = 'u', long)]
        template_uri: Option<String>,
        #[arg(short, long)]
        parameters: Option<String>,
    },
    WhatIf {
        #[arg(short, long)]
        name: Option<String>,
        #[arg(short, long)]
        management_group_id: String,
        #[arg(short, long)]
        location: String,
        #[arg(short = 'f', long)]
        template_file: Option<String>,
        #[arg(short = 'u', long)]
        template_uri: Option<String>,
        #[arg(short, long)]
        parameters: Option<String>,
        #[arg(long, default_value = "FullResourcePayloads")]
        result_format: String,
    },
    Cancel {
        #[arg(short, long)]
        name: String,
        #[arg(short, long)]
        management_group_id: String,
    },
    Wait {
        #[arg(short, long)]
        name: String,
        #[arg(short, long)]
        management_group_id: String,
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
enum DeploymentTenantCommand {
    List,
    Show {
        #[arg(short, long)]
        name: String,
    },
    Export {
        #[arg(short, long)]
        name: String,
    },
    Create {
        #[arg(short, long)]
        name: String,
        #[arg(short, long)]
        location: String,
        #[arg(short = 'f', long)]
        template_file: Option<String>,
        #[arg(short = 'u', long)]
        template_uri: Option<String>,
        #[arg(short, long)]
        parameters: Option<String>,
    },
    Delete {
        #[arg(short, long)]
        name: String,
    },
    Validate {
        #[arg(short, long)]
        name: Option<String>,
        #[arg(short, long)]
        location: String,
        #[arg(short = 'f', long)]
        template_file: Option<String>,
        #[arg(short = 'u', long)]
        template_uri: Option<String>,
        #[arg(short, long)]
        parameters: Option<String>,
    },
    WhatIf {
        #[arg(short, long)]
        name: Option<String>,
        #[arg(short, long)]
        location: String,
        #[arg(short = 'f', long)]
        template_file: Option<String>,
        #[arg(short = 'u', long)]
        template_uri: Option<String>,
        #[arg(short, long)]
        parameters: Option<String>,
        #[arg(long, default_value = "FullResourcePayloads")]
        result_format: String,
    },
    Cancel {
        #[arg(short, long)]
        name: String,
    },
    Wait {
        #[arg(short, long)]
        name: String,
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
    Sub {
        #[command(subcommand)]
        command: DeploymentOperationSubCommand,
    },
    Mg {
        #[command(subcommand)]
        command: DeploymentOperationMgCommand,
    },
    Tenant {
        #[command(subcommand)]
        command: DeploymentOperationTenantCommand,
    },
}

#[derive(Subcommand)]
enum DeploymentOperationGroupCommand {
    List {
        #[arg(short, long)]
        name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
    },
    Show {
        #[arg(short, long)]
        name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
        #[arg(long)]
        operation_id: String,
    },
}

#[derive(Subcommand)]
enum DeploymentOperationSubCommand {
    List {
        #[arg(short, long)]
        name: String,
    },
    Show {
        #[arg(short, long)]
        name: String,
        #[arg(long)]
        operation_id: String,
    },
}

#[derive(Subcommand)]
enum DeploymentOperationMgCommand {
    List {
        #[arg(short, long)]
        name: String,
        #[arg(short, long)]
        management_group_id: String,
    },
    Show {
        #[arg(short, long)]
        name: String,
        #[arg(short, long)]
        management_group_id: String,
        #[arg(long)]
        operation_id: String,
    },
}

#[derive(Subcommand)]
enum DeploymentOperationTenantCommand {
    List {
        #[arg(short, long)]
        name: String,
    },
    Show {
        #[arg(short, long)]
        name: String,
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
    Vnet {
        #[command(subcommand)]
        command: VnetCommand,
    },
    Nsg {
        #[command(subcommand)]
        command: NsgCommand,
    },
    PublicIp {
        #[command(subcommand)]
        command: PublicIpCommand,
    },
    Nic {
        #[command(subcommand)]
        command: NicCommand,
    },
    PrivateEndpoint {
        #[command(subcommand)]
        command: PrivateEndpointCommand,
    },
    Lb {
        #[command(subcommand)]
        command: LoadBalancerCommand,
    },
    RouteTable {
        #[command(subcommand)]
        command: RouteTableCommand,
    },
    Dns {
        #[command(subcommand)]
        command: DnsCommand,
    },
    Watcher {
        #[command(subcommand)]
        command: WatcherCommand,
    },
}

#[derive(Subcommand)]
enum VnetCommand {
    List {
        #[arg(short = 'g', long)]
        resource_group: Option<String>,
    },
    Show {
        #[arg(short, long)]
        name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
    },
    Subnet {
        #[command(subcommand)]
        command: VnetSubnetCommand,
    },
    Peering {
        #[command(subcommand)]
        command: VnetPeeringCommand,
    },
}

#[derive(Subcommand)]
enum VnetSubnetCommand {
    List {
        #[arg(short = 'g', long)]
        resource_group: String,
        #[arg(long)]
        vnet_name: String,
    },
    Show {
        #[arg(short, long)]
        name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
        #[arg(long)]
        vnet_name: String,
    },
}

#[derive(Subcommand)]
enum VnetPeeringCommand {
    List {
        #[arg(short = 'g', long)]
        resource_group: String,
        #[arg(long)]
        vnet_name: String,
    },
    Show {
        #[arg(short, long)]
        name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
        #[arg(long)]
        vnet_name: String,
    },
}

#[derive(Subcommand)]
enum NsgCommand {
    List {
        #[arg(short = 'g', long)]
        resource_group: Option<String>,
    },
    Show {
        #[arg(short, long)]
        name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
    },
    Rule {
        #[command(subcommand)]
        command: NsgRuleCommand,
    },
}

#[derive(Subcommand)]
enum NsgRuleCommand {
    List {
        #[arg(short = 'g', long)]
        resource_group: String,
        #[arg(long)]
        nsg_name: String,
    },
    Show {
        #[arg(short, long)]
        name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
        #[arg(long)]
        nsg_name: String,
    },
}

#[derive(Subcommand)]
enum PublicIpCommand {
    List {
        #[arg(short = 'g', long)]
        resource_group: Option<String>,
    },
    Show {
        #[arg(short, long)]
        name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
    },
}

#[derive(Subcommand)]
enum NicCommand {
    List {
        #[arg(short = 'g', long)]
        resource_group: Option<String>,
    },
    Show {
        #[arg(short, long)]
        name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
    },
    IpConfig {
        #[command(subcommand)]
        command: NicIpConfigCommand,
    },
}

#[derive(Subcommand)]
enum NicIpConfigCommand {
    List {
        #[arg(short = 'g', long)]
        resource_group: String,
        #[arg(long)]
        nic_name: String,
    },
    Show {
        #[arg(short, long)]
        name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
        #[arg(long)]
        nic_name: String,
    },
}

#[derive(Subcommand)]
enum PrivateEndpointCommand {
    List {
        #[arg(short = 'g', long)]
        resource_group: Option<String>,
    },
    Show {
        #[arg(short, long)]
        name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
    },
}

#[derive(Subcommand)]
enum LoadBalancerCommand {
    List {
        #[arg(short = 'g', long)]
        resource_group: Option<String>,
    },
    Show {
        #[arg(short, long)]
        name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
    },
    ListMapping {
        #[arg(short, long)]
        name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
    },
    ListNic {
        #[arg(short, long)]
        name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
    },
    AddressPool {
        #[command(subcommand)]
        command: LoadBalancerAddressPoolCommand,
    },
    FrontendIp {
        #[command(subcommand)]
        command: LoadBalancerFrontendIpCommand,
    },
    InboundNatPool {
        #[command(subcommand)]
        command: LoadBalancerInboundNatPoolCommand,
    },
    InboundNatRule {
        #[command(subcommand)]
        command: LoadBalancerInboundNatRuleCommand,
    },
    OutboundRule {
        #[command(subcommand)]
        command: LoadBalancerOutboundRuleCommand,
    },
    Probe {
        #[command(subcommand)]
        command: LoadBalancerProbeCommand,
    },
    Rule {
        #[command(subcommand)]
        command: LoadBalancerRuleCommand,
    },
}

#[derive(Subcommand)]
enum LoadBalancerAddressPoolCommand {
    List {
        #[arg(short, long)]
        lb_name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
    },
    Show {
        #[arg(short, long)]
        name: String,
        #[arg(short, long)]
        lb_name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
    },
}

#[derive(Subcommand)]
enum LoadBalancerFrontendIpCommand {
    List {
        #[arg(short, long)]
        lb_name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
    },
    Show {
        #[arg(short, long)]
        name: String,
        #[arg(short, long)]
        lb_name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
    },
}

#[derive(Subcommand)]
enum LoadBalancerInboundNatPoolCommand {
    List {
        #[arg(short, long)]
        lb_name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
    },
    Show {
        #[arg(short, long)]
        name: String,
        #[arg(short, long)]
        lb_name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
    },
}

#[derive(Subcommand)]
enum LoadBalancerInboundNatRuleCommand {
    List {
        #[arg(short, long)]
        lb_name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
    },
    Show {
        #[arg(short, long)]
        name: String,
        #[arg(short, long)]
        lb_name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
    },
}

#[derive(Subcommand)]
enum LoadBalancerOutboundRuleCommand {
    List {
        #[arg(short, long)]
        lb_name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
    },
    Show {
        #[arg(short, long)]
        name: String,
        #[arg(short, long)]
        lb_name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
    },
}

#[derive(Subcommand)]
enum LoadBalancerProbeCommand {
    List {
        #[arg(short, long)]
        lb_name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
    },
    Show {
        #[arg(short, long)]
        name: String,
        #[arg(short, long)]
        lb_name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
    },
}

#[derive(Subcommand)]
enum LoadBalancerRuleCommand {
    List {
        #[arg(short, long)]
        lb_name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
    },
    Show {
        #[arg(short, long)]
        name: String,
        #[arg(short, long)]
        lb_name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
    },
}

#[derive(Subcommand)]
enum RouteTableCommand {
    List {
        #[arg(short = 'g', long)]
        resource_group: Option<String>,
    },
    Show {
        #[arg(short, long)]
        name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
    },
    Route {
        #[command(subcommand)]
        command: RouteTableRouteCommand,
    },
}

#[derive(Subcommand)]
enum RouteTableRouteCommand {
    List {
        #[arg(short, long)]
        table_name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
    },
    Show {
        #[arg(short, long)]
        name: String,
        #[arg(short, long)]
        table_name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
    },
}

#[derive(Subcommand)]
enum DnsCommand {
    ListZone {
        #[arg(short = 'g', long)]
        resource_group: Option<String>,
    },
    ShowZone {
        #[arg(short, long)]
        name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
    },
    RecordSet {
        #[command(subcommand)]
        command: DnsRecordSetCommand,
    },
}

#[derive(Subcommand)]
enum DnsRecordSetCommand {
    List {
        #[arg(short, long)]
        zone_name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
        #[arg(short, long, default_value = "*")]
        record_type: String,
    },
    Show {
        #[arg(short, long)]
        name: String,
        #[arg(short, long)]
        zone_name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
        #[arg(short, long)]
        record_type: String,
    },
}


#[derive(Subcommand)]
enum WatcherCommand {
    List,
    Show {
        #[arg(short, long)]
        name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
    },
    ConnectionMonitor {
        #[command(subcommand)]
        command: WatcherConnectionMonitorCommand,
    },
    FlowLog {
        #[command(subcommand)]
        command: WatcherFlowLogCommand,
    },
    PacketCapture {
        #[command(subcommand)]
        command: WatcherPacketCaptureCommand,
    },
}

#[derive(Subcommand)]
enum WatcherConnectionMonitorCommand {
    List {
        #[arg(short, long)]
        watcher_name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
    },
    Show {
        #[arg(short, long)]
        name: String,
        #[arg(short, long)]
        watcher_name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
    },
}

#[derive(Subcommand)]
enum WatcherFlowLogCommand {
    List {
        #[arg(short, long)]
        watcher_name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
    },
    Show {
        #[arg(short, long)]
        name: String,
        #[arg(short, long)]
        watcher_name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
    },
}

#[derive(Subcommand)]
enum WatcherPacketCaptureCommand {
    List {
        #[arg(short, long)]
        watcher_name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
    },
    Show {
        #[arg(short, long)]
        name: String,
        #[arg(short, long)]
        watcher_name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
    },
}




#[derive(Subcommand)]
enum BastionCommand {
    Create {
        #[arg(short, long)]
        name: String,
        #[arg(short = 'g', long)]
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
        #[arg(short = 'g', long)]
        resource_group: String,
    },
    List {
        #[arg(short = 'g', long)]
        resource_group: Option<String>,
    },
    Show {
        #[arg(short, long)]
        name: String,
        #[arg(short = 'g', long)]
        resource_group: String,
    },
    Update {
        #[arg(short, long)]
        name: String,
        #[arg(short = 'g', long)]
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
        #[arg(short = 'g', long)]
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
        #[arg(short = 'g', long)]
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
        #[arg(short = 'g', long)]
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
        #[arg(short = 'g', long)]
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
    let query = cli.query;

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
            AccountCommand::Show { name } => {
                let provider = auth::TokenProvider::load(subscription)?;
                let value = commands::account::show::execute(&provider, name.as_deref())?;
                output::print_output(&value, output_format, query.as_deref())
            }
            AccountCommand::List => {
                let mut provider = auth::TokenProvider::load(subscription)?;
                let value = commands::account::list::execute(&mut provider).await?;
                output::print_output(&value, output_format, query.as_deref())
            }
            AccountCommand::Set { name } => {
                let mut provider = auth::TokenProvider::load(subscription)?;
                let value = commands::account::set::execute(&mut provider, &name).await?;
                output::print_output(&value, output_format, query.as_deref())
            }
            AccountCommand::ListLocations { subscription_id } => {
                let mut provider = auth::TokenProvider::load(subscription.clone())?;
                let token = provider.get_access_token().await?;
                let sub = subscription_id
                    .or(subscription)
                    .or_else(|| provider.cache_default_subscription())
                    .ok_or_else(|| anyhow::anyhow!("no subscription specified and no default in cache"))?;
                let client = arm_client::ArmClient::new(token, sub.clone());
                let value = commands::account::list_locations::execute(&client, Some(&sub)).await?;
                output::print_output(&value, output_format, query.as_deref())
            }
            AccountCommand::GetAccessToken => {
                let mut provider = auth::TokenProvider::load(subscription)?;
                let value = commands::account::get_access_token::execute(&mut provider).await?;
                output::print_output(&value, output_format, query.as_deref())
            }
            AccountCommand::Clear => {
                let mut provider = auth::TokenProvider::load(subscription)?;
                let value = commands::account::clear::execute(&mut provider)?;
                output::print_output(&value, output_format, query.as_deref())
            }
        },

        CliCommand::Role { command } => {
            handle_role(command, output_format, subscription, query.as_deref()).await
        }

        CliCommand::Group { command } => {
            handle_group(command, output_format, subscription, query.as_deref()).await
        }

        CliCommand::Vm { command } => {
            handle_vm(command, output_format, subscription, query.as_deref()).await
        }

        CliCommand::Vmss { command } => {
            handle_vmss(command, output_format, subscription, query.as_deref()).await
        }

        CliCommand::Disk { command } => {
            handle_disk(command, output_format, subscription, query.as_deref()).await
        }

        CliCommand::Image { command } => {
            handle_image(command, output_format, subscription, query.as_deref()).await
        }

        CliCommand::Sig { command } => {
            handle_sig(command, output_format, subscription, query.as_deref()).await
        }

        CliCommand::Deployment { command } => {
            handle_deployment(command, output_format, subscription, query.as_deref()).await
        }

        CliCommand::Network { command } => match command {
            NetworkCommand::Bastion { command } => {
                handle_bastion(command, output_format, subscription, query.as_deref()).await?;
                Ok(())
            }
            NetworkCommand::Vnet { command } => {
                handle_network_vnet(command, output_format, subscription, query.as_deref()).await
            }
            NetworkCommand::Nsg { command } => {
                handle_network_nsg(command, output_format, subscription, query.as_deref()).await
            }
            NetworkCommand::PublicIp { command } => {
                handle_network_public_ip(command, output_format, subscription, query.as_deref()).await
            }
            NetworkCommand::Nic { command } => {
                handle_network_nic(command, output_format, subscription, query.as_deref()).await
            }
            NetworkCommand::PrivateEndpoint { command } => {
                handle_network_private_endpoint(command, output_format, subscription, query.as_deref()).await
            }
            NetworkCommand::Lb { command } => {
                handle_network_lb(command, output_format, subscription, query.as_deref()).await
            }
            NetworkCommand::RouteTable { command } => {
                handle_network_route_table(command, output_format, subscription, query.as_deref()).await
            }
            NetworkCommand::Dns { command } => {
                handle_network_dns(command, output_format, subscription, query.as_deref()).await
            }
            NetworkCommand::Watcher { command } => {
                handle_network_watcher(command, output_format, subscription, query.as_deref()).await
            }
        }

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
                output::print_output(&value, output_format, query.as_deref())
            }
        }
    }
}

async fn handle_group(
    cmd: GroupCommand,
    output_format: OutputFormat,
    subscription: Option<String>,
    query: Option<&str>,
) -> anyhow::Result<()> {
    let mut provider = auth::TokenProvider::load(subscription)?;
    let access_token = provider.get_access_token().await?;
    let subscription_id = provider.get_subscription_id_or_fallback().await?;

    let client = arm_client::ArmClient::new(access_token, subscription_id);

    match cmd {
        GroupCommand::List => {
            let value = commands::group::list::execute(&client).await?;
            output::print_output(&value, output_format, query)
        }
        GroupCommand::Show { name } => {
            let value = commands::group::show::execute(&client, &name).await?;
            output::print_output(&value, output_format, query)
        }
    }
}

async fn handle_vm(
    cmd: VmCommand,
    output_format: OutputFormat,
    subscription: Option<String>,
    query: Option<&str>,
) -> anyhow::Result<()> {
    let mut provider = auth::TokenProvider::load(subscription)?;
    let access_token = provider.get_access_token().await?;
    let subscription_id = provider.get_subscription_id_or_fallback().await?;

    let client = arm_client::ArmClient::new(access_token, subscription_id);

    match cmd {
        VmCommand::List { resource_group } => {
            let value = commands::vm::list::execute(&client, resource_group.as_deref()).await?;
            output::print_output(&value, output_format, query)
        }
        VmCommand::Show { name, resource_group } => {
            let value = commands::vm::show::execute(&client, &resource_group, &name).await?;
            output::print_output(&value, output_format, query)
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
        VmCommand::GetInstanceView { name, resource_group } => {
            let value = commands::vm::get_instance_view::execute(&client, &resource_group, &name).await?;
            output::print_output(&value, output_format, query)
        }
        VmCommand::ListIpAddresses { name, resource_group } => {
            let value = commands::vm::list_ip_addresses::execute(&client, resource_group.as_deref(), name.as_deref()).await?;
            output::print_output(&value, output_format, query)
        }
        VmCommand::ListSizes { location } => {
            let value = commands::vm::list_sizes::execute(&client, &location).await?;
            output::print_output(&value, output_format, query)
        }
        VmCommand::ListSkus { location, resource_type, size, zone } => {
            let value = commands::vm::list_skus::execute(&client, location.as_deref(), resource_type.as_deref(), size.as_deref(), zone).await?;
            output::print_output(&value, output_format, query)
        }
        VmCommand::ListUsage { location } => {
            let value = commands::vm::list_usage::execute(&client, &location).await?;
            output::print_output(&value, output_format, query)
        }
        VmCommand::ListVmResizeOptions { name, resource_group } => {
            let value = commands::vm::list_vm_resize_options::execute(&client, &resource_group, &name).await?;
            output::print_output(&value, output_format, query)
        }
        VmCommand::Restart { name, resource_group, no_wait } => {
            commands::vm::restart::execute(&client, &resource_group, &name, no_wait).await
        }
        VmCommand::Create { name, resource_group, image, size, location, admin_username, admin_password, ssh_key_value, authentication_type, subnet, os_disk_size_gb, data_disk_sizes_gb, tags, no_wait: _ } => {
            let loc = location.as_deref().unwrap_or("eastus");
            let tags_ref = if tags.is_empty() { None } else { Some(tags.as_slice()) };
            let value = commands::vm::create::execute(
                &client, &resource_group, &name, &image, loc, &size,
                admin_username.as_deref(), admin_password.as_deref(),
                ssh_key_value.as_deref(), authentication_type.as_deref(),
                subnet.as_deref(), os_disk_size_gb, &data_disk_sizes_gb, tags_ref,
            ).await?;
            output::print_output(&value, output_format, query)
        }
        VmCommand::Delete { name, resource_group, force_deletion, no_wait } => {
            commands::vm::delete_vm::execute(&client, &resource_group, &name, force_deletion, no_wait).await
        }
        VmCommand::Update { name, resource_group, set, no_wait: _ } => {
            let value = commands::vm::update_vm::execute(&client, &resource_group, &name, &set).await?;
            output::print_output(&value, output_format, query)
        }
        VmCommand::Resize { name, resource_group, size, no_wait: _ } => {
            let value = commands::vm::resize::execute(&client, &resource_group, &name, &size).await?;
            output::print_output(&value, output_format, query)
        }
        VmCommand::Redeploy { name, resource_group, no_wait } => {
            commands::vm::redeploy::execute(&client, &resource_group, &name, no_wait).await
        }
        VmCommand::Reimage { name, resource_group, no_wait } => {
            commands::vm::reimage::execute(&client, &resource_group, &name, no_wait).await
        }
        VmCommand::Reapply { name, resource_group, no_wait } => {
            commands::vm::reapply::execute(&client, &resource_group, &name, no_wait).await
        }
        VmCommand::PerformMaintenance { name, resource_group } => {
            commands::vm::perform_maintenance::execute(&client, &resource_group, &name).await
        }
        VmCommand::SimulateEviction { name, resource_group } => {
            commands::vm::simulate_eviction::execute(&client, &resource_group, &name).await
        }
        VmCommand::Generalize { name, resource_group } => {
            commands::vm::generalize::execute(&client, &resource_group, &name).await
        }
        VmCommand::Capture { name, resource_group, vhd_name_prefix, storage_container, overwrite } => {
            let value = commands::vm::capture::execute(&client, &resource_group, &name, &vhd_name_prefix, &storage_container, overwrite).await?;
            output::print_output(&value, output_format, query)
        }
        VmCommand::Convert { name, resource_group } => {
            commands::vm::convert::execute(&client, &resource_group, &name).await
        }
        VmCommand::AssessPatches { name, resource_group } => {
            let value = commands::vm::assess_patches::execute(&client, &resource_group, &name).await?;
            output::print_output(&value, output_format, query)
        }
        VmCommand::InstallPatches { name, resource_group, maximum_duration, reboot_setting, classifications_to_include_linux, classifications_to_include_win } => {
            let linux_cls = if classifications_to_include_linux.is_empty() { None } else { Some(classifications_to_include_linux.as_slice()) };
            let win_cls = if classifications_to_include_win.is_empty() { None } else { Some(classifications_to_include_win.as_slice()) };
            let value = commands::vm::install_patches::execute(&client, &resource_group, &name, &maximum_duration, &reboot_setting, linux_cls, win_cls).await?;
            output::print_output(&value, output_format, query)
        }
        VmCommand::AutoShutdown { name, resource_group, time, off, email, webhook, location } => {
            let loc = location.as_deref().unwrap_or("eastus");
            let value = commands::vm::auto_shutdown::execute(&client, &resource_group, &name, time.as_deref(), off, email.as_deref(), webhook.as_deref(), loc).await?;
            output::print_output(&value, output_format, query)
        }
        VmCommand::OpenPort { name, resource_group, port, priority, nsg_name, apply_to_subnet } => {
            let value = commands::vm::open_port::execute(&client, &resource_group, &name, &port, priority, nsg_name.as_deref(), apply_to_subnet).await?;
            output::print_output(&value, output_format, query)
        }
        VmCommand::Disk { command } => match command {
            VmDiskCommand::Attach { vm_name, resource_group, name, new, size_gb, sku, lun, caching, enable_write_accelerator } => {
                let value = commands::vm::disk::attach::execute(&client, &resource_group, &vm_name, &name, new, size_gb, sku.as_deref(), lun, caching.as_deref(), enable_write_accelerator).await?;
                output::print_output(&value, output_format, query)
            }
            VmDiskCommand::Detach { vm_name, resource_group, name, force_detach } => {
                let value = commands::vm::disk::detach::execute(&client, &resource_group, &vm_name, &name, force_detach).await?;
                output::print_output(&value, output_format, query)
            }
        },
        VmCommand::Nic { command } => match command {
            VmNicCommand::List { vm_name, resource_group } => {
                let value = commands::vm::nic::list::execute(&client, &resource_group, &vm_name).await?;
                output::print_output(&value, output_format, query)
            }
            VmNicCommand::Show { vm_name, resource_group, nic } => {
                let value = commands::vm::nic::show::execute(&client, &resource_group, &vm_name, &nic).await?;
                output::print_output(&value, output_format, query)
            }
            VmNicCommand::Add { vm_name, resource_group, nics, primary_nic } => {
                let value = commands::vm::nic::add::execute(&client, &resource_group, &vm_name, &nics, primary_nic.as_deref()).await?;
                output::print_output(&value, output_format, query)
            }
            VmNicCommand::Remove { vm_name, resource_group, nics, primary_nic } => {
                let value = commands::vm::nic::remove::execute(&client, &resource_group, &vm_name, &nics, primary_nic.as_deref()).await?;
                output::print_output(&value, output_format, query)
            }
            VmNicCommand::Set { vm_name, resource_group, nics, primary_nic } => {
                let value = commands::vm::nic::set::execute(&client, &resource_group, &vm_name, &nics, primary_nic.as_deref()).await?;
                output::print_output(&value, output_format, query)
            }
        },
        VmCommand::RunCommand { command } => match command {
            VmRunCommandCommand::Invoke { vm_name, resource_group, command_id, scripts, parameters } => {
                let value = commands::vm::run_command::invoke::execute(&client, &resource_group, &vm_name, &command_id, &scripts, &parameters).await?;
                output::print_output(&value, output_format, query)
            }
            VmRunCommandCommand::List { vm_name, resource_group, location, expand_instance_view } => {
                let value = commands::vm::run_command::list::execute(&client, resource_group.as_deref(), vm_name.as_deref(), location.as_deref(), expand_instance_view).await?;
                output::print_output(&value, output_format, query)
            }
            VmRunCommandCommand::Show { vm_name, resource_group, name, location, command_id, instance_view } => {
                let value = commands::vm::run_command::show::execute(&client, resource_group.as_deref(), vm_name.as_deref(), name.as_deref(), location.as_deref(), command_id.as_deref(), instance_view).await?;
                output::print_output(&value, output_format, query)
            }
            VmRunCommandCommand::Create { vm_name, resource_group, name, location, script, script_uri, command_id, parameters, protected_parameters, run_as_user, run_as_password, async_execution, timeout_in_seconds, output_blob_uri, error_blob_uri } => {
                let value = commands::vm::run_command::create::execute(&client, &resource_group, &vm_name, &name, location.as_deref(), script.as_deref(), script_uri.as_deref(), command_id.as_deref(), &parameters, &protected_parameters, run_as_user.as_deref(), run_as_password.as_deref(), async_execution, timeout_in_seconds, output_blob_uri.as_deref(), error_blob_uri.as_deref()).await?;
                output::print_output(&value, output_format, query)
            }
            VmRunCommandCommand::Update { vm_name, resource_group, name, script, script_uri, command_id, parameters, protected_parameters, run_as_user, run_as_password, timeout_in_seconds, output_blob_uri, error_blob_uri } => {
                let value = commands::vm::run_command::update::execute(&client, &resource_group, &vm_name, &name, script.as_deref(), script_uri.as_deref(), command_id.as_deref(), &parameters, &protected_parameters, run_as_user.as_deref(), run_as_password.as_deref(), timeout_in_seconds, output_blob_uri.as_deref(), error_blob_uri.as_deref()).await?;
                output::print_output(&value, output_format, query)
            }
            VmRunCommandCommand::Delete { vm_name, resource_group, name } => {
                let value = commands::vm::run_command::delete::execute(&client, &resource_group, &vm_name, &name).await?;
                output::print_output(&value, output_format, query)
            }
        },
        VmCommand::Wait { name, resource_group, created, updated, deleted, exists, interval, timeout } => {
            commands::vm::vm_wait::execute(&client, &resource_group, &name, created, updated, deleted, exists, interval, timeout).await
        }
    }
}

async fn handle_vmss(
    cmd: VmssCommand,
    output_format: OutputFormat,
    subscription: Option<String>,
    query: Option<&str>,
) -> anyhow::Result<()> {
    let mut provider = auth::TokenProvider::load(subscription)?;
    let access_token = provider.get_access_token().await?;
    let subscription_id = provider.get_subscription_id_or_fallback().await?;

    let client = arm_client::ArmClient::new(access_token, subscription_id);

    match cmd {
        VmssCommand::List { resource_group } => {
            let value = commands::vmss::list::execute(&client, resource_group.as_deref()).await?;
            output::print_output(&value, output_format, query)
        }
        VmssCommand::Show { name, resource_group } => {
            let value = commands::vmss::show::execute(&client, &resource_group, &name).await?;
            output::print_output(&value, output_format, query)
        }
        VmssCommand::ListInstances { name, resource_group, expand } => {
            let value = commands::vmss::list_instances::execute(&client, &resource_group, &name, expand.as_deref()).await?;
            output::print_output(&value, output_format, query)
        }
        VmssCommand::ListSkus { name, resource_group } => {
            let value = commands::vmss::list_skus::execute(&client, &resource_group, &name).await?;
            output::print_output(&value, output_format, query)
        }
        VmssCommand::ListInstancePublicIps { name, resource_group } => {
            let value = commands::vmss::list_instance_public_ips::execute(&client, &resource_group, &name).await?;
            output::print_output(&value, output_format, query)
        }
        VmssCommand::ListInstanceConnectionInfo { name, resource_group } => {
            let value = commands::vmss::list_instance_connection_info::execute(&client, &resource_group, &name).await?;
            output::print_output(&value, output_format, query)
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

async fn handle_role(
    cmd: RoleCommand,
    output_format: OutputFormat,
    subscription: Option<String>,
    query: Option<&str>,
) -> anyhow::Result<()> {
    let mut provider = auth::TokenProvider::load(subscription.clone())?;
    let access_token = provider.get_access_token().await?;
    let subscription_id = provider.get_subscription_id_or_fallback().await?;
    let client = arm_client::ArmClient::new(access_token, subscription_id);

    match cmd {
        RoleCommand::Assignment { command } => match command {
            RoleAssignmentCommand::List { assignee, role, scope, resource_group, include_groups, all } => {
                let value = commands::role::assignment::list::execute(
                    &client,
                    assignee.as_deref(),
                    role.as_deref(),
                    scope.as_deref(),
                    resource_group.as_deref(),
                    subscription.as_deref(),
                    include_groups,
                    all,
                )
                .await?;
                output::print_output(&value, output_format, query)
            }
            RoleAssignmentCommand::Show { ids, name, scope } => {
                let value = commands::role::assignment::show::execute(
                    &client,
                    ids.as_deref(),
                    name.as_deref(),
                    scope.as_deref(),
                    subscription.as_deref(),
                )
                .await?;
                output::print_output(&value, output_format, query)
            }
        },
        RoleCommand::Definition { command } => match command {
            RoleDefinitionCommand::List { name, scope, custom_role_only } => {
                let value = commands::role::definition::list::execute(
                    &client,
                    name.as_deref(),
                    scope.as_deref(),
                    subscription.as_deref(),
                    custom_role_only,
                )
                .await?;
                output::print_output(&value, output_format, query)
            }
            RoleDefinitionCommand::Show { name, scope } => {
                let value = commands::role::definition::show::execute(
                    &client,
                    &name,
                    scope.as_deref(),
                    subscription.as_deref(),
                )
                .await?;
                output::print_output(&value, output_format, query)
            }
        },
        RoleCommand::Pim { command } => match command {
            RolePimCommand::List { scope } => {
                let value = commands::role::pim::list::execute(
                    &client,
                    scope.as_deref(),
                    subscription.as_deref(),
                )
                .await?;
                output::print_output(&value, output_format, query)
            }
            RolePimCommand::Status { scope } => {
                let value = commands::role::pim::status::execute(
                    &client,
                    scope.as_deref(),
                    subscription.as_deref(),
                )
                .await?;
                output::print_output(&value, output_format, query)
            }
            RolePimCommand::Activate { role, justification, duration, scope } => {
                let value = commands::role::pim::activate::execute(
                    &client,
                    &role,
                    &justification,
                    &duration,
                    scope.as_deref(),
                    subscription.as_deref(),
                )
                .await?;
                output::print_output(&value, output_format, query)
            }
            RolePimCommand::Deactivate { role, scope } => {
                let value = commands::role::pim::deactivate::execute(
                    &client,
                    &role,
                    scope.as_deref(),
                    subscription.as_deref(),
                )
                .await?;
                output::print_output(&value, output_format, query)
            }
        },
    }
}

async fn handle_disk(
    cmd: DiskCommand,
    output_format: OutputFormat,
    subscription: Option<String>,
    query: Option<&str>,
) -> anyhow::Result<()> {
    let mut provider = auth::TokenProvider::load(subscription)?;
    let access_token = provider.get_access_token().await?;
    let subscription_id = provider.get_subscription_id_or_fallback().await?;

    let client = arm_client::ArmClient::new(access_token, subscription_id);

    match cmd {
        DiskCommand::List { resource_group } => {
            let value = commands::disk::list::execute(&client, resource_group.as_deref()).await?;
            output::print_output(&value, output_format, query)
        }
        DiskCommand::Show { name, resource_group } => {
            let value = commands::disk::show::execute(&client, &resource_group, &name).await?;
            output::print_output(&value, output_format, query)
        }
        DiskCommand::ListSkus { location, zone } => {
            let value = commands::disk::list_skus::execute(&client, location.as_deref(), zone).await?;
            output::print_output(&value, output_format, query)
        }
        DiskCommand::Create { name, resource_group, location, size_gb, sku, source, zone, hyper_v_generation, os_type } => {
            let loc = location.as_deref().unwrap_or("eastus");
            let value = commands::disk::create::execute(
                &client, &resource_group, &name, loc,
                size_gb, sku.as_deref(), source.as_deref(),
                zone.as_deref(), hyper_v_generation.as_deref(), os_type.as_deref(),
            ).await?;
            output::print_output(&value, output_format, query)
        }
        DiskCommand::Delete { name, resource_group } => {
            commands::disk::delete::execute(&client, &resource_group, &name).await
        }
        DiskCommand::Update { name, resource_group, size_gb, sku } => {
            let value = commands::disk::update::execute(&client, &resource_group, &name, size_gb, sku.as_deref()).await?;
            output::print_output(&value, output_format, query)
        }
        DiskCommand::GrantAccess { name, resource_group, access_level, duration_in_seconds } => {
            let value = commands::disk::grant_access::execute(&client, &resource_group, &name, &access_level, duration_in_seconds).await?;
            output::print_output(&value, output_format, query)
        }
        DiskCommand::RevokeAccess { name, resource_group } => {
            commands::disk::revoke_access::execute(&client, &resource_group, &name).await
        }
    }
}

async fn handle_image(
    cmd: ImageCommand,
    output_format: OutputFormat,
    subscription: Option<String>,
    query: Option<&str>,
) -> anyhow::Result<()> {
    let mut provider = auth::TokenProvider::load(subscription)?;
    let access_token = provider.get_access_token().await?;
    let subscription_id = provider.get_subscription_id_or_fallback().await?;

    let client = arm_client::ArmClient::new(access_token, subscription_id);

    match cmd {
        ImageCommand::List { resource_group } => {
            let value = commands::image::list::execute(&client, resource_group.as_deref()).await?;
            output::print_output(&value, output_format, query)
        }
        ImageCommand::Show { name, resource_group } => {
            let value = commands::image::show::execute(&client, &resource_group, &name).await?;
            output::print_output(&value, output_format, query)
        }
        ImageCommand::Builder { command } => match command {
            ImageBuilderCommand::List { resource_group } => {
                let value =
                    commands::image::builder::list::execute(&client, resource_group.as_deref())
                        .await?;
                output::print_output(&value, output_format, query)
            }
            ImageBuilderCommand::Show { name, resource_group } => {
                let value =
                    commands::image::builder::show::execute(&client, &resource_group, &name)
                        .await?;
                output::print_output(&value, output_format, query)
            }
            ImageBuilderCommand::ShowRuns { name, resource_group, output_name } => {
                let value = commands::image::builder::show_runs::execute(
                    &client,
                    &resource_group,
                    &name,
                    output_name.as_deref(),
                )
                .await?;
                output::print_output(&value, output_format, query)
            }
        },
    }
}

async fn handle_sig(
    cmd: SigCommand,
    output_format: OutputFormat,
    subscription: Option<String>,
    query: Option<&str>,
) -> anyhow::Result<()> {
    let mut provider = auth::TokenProvider::load(subscription)?;
    let access_token = provider.get_access_token().await?;
    let subscription_id = provider.get_subscription_id_or_fallback().await?;

    let client = arm_client::ArmClient::new(access_token, subscription_id);

    match cmd {
        SigCommand::List { resource_group } => {
            let value = commands::sig::list::execute(&client, resource_group.as_deref()).await?;
            output::print_output(&value, output_format, query)
        }
        SigCommand::Show { gallery_name, resource_group } => {
            let value = commands::sig::show::execute(&client, &resource_group, &gallery_name).await?;
            output::print_output(&value, output_format, query)
        }
        SigCommand::ListShared { location, shared_to } => {
            let to_tenant = shared_to.as_deref() == Some("tenant");
            let value = commands::sig::list_shared::execute(&client, &location, to_tenant).await?;
            output::print_output(&value, output_format, query)
        }
        SigCommand::ShowShared { location, gallery_unique_name } => {
            let value = commands::sig::show_shared::execute(&client, &location, &gallery_unique_name).await?;
            output::print_output(&value, output_format, query)
        }
        SigCommand::ListCommunity { location } => {
            let value = commands::sig::list_community::execute(&client, location.as_deref()).await?;
            output::print_output(&value, output_format, query)
        }
        SigCommand::ShowCommunity { location, public_gallery_name } => {
            let value = commands::sig::show_community::execute(&client, &location, &public_gallery_name).await?;
            output::print_output(&value, output_format, query)
        }
        SigCommand::ImageDefinition { command } => match command {
            SigImageDefinitionCommand::List { resource_group, gallery_name } => {
                let value = commands::sig::image_definition::list::execute(&client, &resource_group, &gallery_name).await?;
                output::print_output(&value, output_format, query)
            }
            SigImageDefinitionCommand::Show { resource_group, gallery_name, gallery_image_definition } => {
                let value = commands::sig::image_definition::show::execute(&client, &resource_group, &gallery_name, &gallery_image_definition).await?;
                output::print_output(&value, output_format, query)
            }
            SigImageDefinitionCommand::ListShared { location, gallery_unique_name } => {
                let value = commands::sig::image_definition::list_shared::execute(&client, &location, &gallery_unique_name).await?;
                output::print_output(&value, output_format, query)
            }
            SigImageDefinitionCommand::ShowShared { location, gallery_unique_name, gallery_image_definition } => {
                let value = commands::sig::image_definition::show_shared::execute(&client, &location, &gallery_unique_name, &gallery_image_definition).await?;
                output::print_output(&value, output_format, query)
            }
            SigImageDefinitionCommand::ListCommunity { location, public_gallery_name } => {
                let value = commands::sig::image_definition::list_community::execute(&client, &location, &public_gallery_name).await?;
                output::print_output(&value, output_format, query)
            }
            SigImageDefinitionCommand::ShowCommunity { location, public_gallery_name, gallery_image_definition } => {
                let value = commands::sig::image_definition::show_community::execute(&client, &location, &public_gallery_name, &gallery_image_definition).await?;
                output::print_output(&value, output_format, query)
            }
        },
        SigCommand::ImageVersion { command } => match command {
            SigImageVersionCommand::List { resource_group, gallery_name, gallery_image_definition } => {
                let value = commands::sig::image_version::list::execute(&client, &resource_group, &gallery_name, &gallery_image_definition).await?;
                output::print_output(&value, output_format, query)
            }
            SigImageVersionCommand::Show { resource_group, gallery_name, gallery_image_definition, gallery_image_version } => {
                let value = commands::sig::image_version::show::execute(&client, &resource_group, &gallery_name, &gallery_image_definition, &gallery_image_version).await?;
                output::print_output(&value, output_format, query)
            }
            SigImageVersionCommand::ListShared { location, gallery_unique_name, gallery_image_definition } => {
                let value = commands::sig::image_version::list_shared::execute(&client, &location, &gallery_unique_name, &gallery_image_definition).await?;
                output::print_output(&value, output_format, query)
            }
            SigImageVersionCommand::ShowShared { location, gallery_unique_name, gallery_image_definition, gallery_image_version } => {
                let value = commands::sig::image_version::show_shared::execute(&client, &location, &gallery_unique_name, &gallery_image_definition, &gallery_image_version).await?;
                output::print_output(&value, output_format, query)
            }
            SigImageVersionCommand::ListCommunity { location, public_gallery_name, gallery_image_definition } => {
                let value = commands::sig::image_version::list_community::execute(&client, &location, &public_gallery_name, &gallery_image_definition).await?;
                output::print_output(&value, output_format, query)
            }
            SigImageVersionCommand::ShowCommunity { location, public_gallery_name, gallery_image_definition, gallery_image_version } => {
                let value = commands::sig::image_version::show_community::execute(&client, &location, &public_gallery_name, &gallery_image_definition, &gallery_image_version).await?;
                output::print_output(&value, output_format, query)
            }
        },
    }
}

async fn handle_deployment(
    cmd: DeploymentCommand,
    output_format: OutputFormat,
    subscription: Option<String>,
    query: Option<&str>,
) -> anyhow::Result<()> {
    let mut provider = auth::TokenProvider::load(subscription)?;
    let access_token = provider.get_access_token().await?;
    let subscription_id = provider.get_subscription_id_or_fallback().await?;

    let client = arm_client::ArmClient::new(access_token, subscription_id);

    match cmd {
        DeploymentCommand::Group { command } => match command {
            DeploymentGroupCommand::List { resource_group } => {
                let value = commands::deployment::group::list::execute(&client, &resource_group).await?;
                output::print_output(&value, output_format, query)
            }
            DeploymentGroupCommand::Show { name, resource_group } => {
                let value = commands::deployment::group::show::execute(&client, &resource_group, &name).await?;
                output::print_output(&value, output_format, query)
            }
            DeploymentGroupCommand::Export { name, resource_group } => {
                let value = commands::deployment::group::export::execute(&client, &resource_group, &name).await?;
                output::print_output(&value, output_format, query)
            }
            DeploymentGroupCommand::Create { resource_group, name, template_file, template_uri, parameters, mode } => {
                let value = commands::deployment::group::create::execute(
                    &client, &resource_group, &name,
                    template_file.as_deref(), template_uri.as_deref(),
                    parameters.as_deref(), &mode,
                ).await?;
                output::print_output(&value, output_format, query)
            }
            DeploymentGroupCommand::Delete { name, resource_group } => {
                commands::deployment::group::delete::execute(&client, &resource_group, &name).await
            }
            DeploymentGroupCommand::Validate { resource_group, name, template_file, template_uri, parameters, mode } => {
                let deploy_name = name.unwrap_or_else(|| "validation".to_string());
                let value = commands::deployment::group::validate::execute(
                    &client, &resource_group, &deploy_name,
                    template_file.as_deref(), template_uri.as_deref(),
                    parameters.as_deref(), &mode,
                ).await?;
                output::print_output(&value, output_format, query)
            }
            DeploymentGroupCommand::WhatIf { resource_group, name, template_file, template_uri, parameters, mode, result_format } => {
                let deploy_name = name.unwrap_or_else(|| "what-if".to_string());
                let value = commands::deployment::group::what_if::execute(
                    &client, &resource_group, &deploy_name,
                    template_file.as_deref(), template_uri.as_deref(),
                    parameters.as_deref(), &mode, Some(&result_format),
                ).await?;
                output::print_output(&value, output_format, query)
            }
            DeploymentGroupCommand::Cancel { name, resource_group } => {
                commands::deployment::group::cancel::execute(&client, &resource_group, &name).await
            }
            DeploymentGroupCommand::Wait { name, resource_group, created, updated, deleted, exists, interval, timeout } => {
                commands::deployment::group::wait::execute(&client, &resource_group, &name, created, updated, deleted, exists, interval, timeout).await
            }
        },
        DeploymentCommand::Sub { command } => match command {
            DeploymentSubCommand::List => {
                let value = commands::deployment::sub::list::execute(&client).await?;
                output::print_output(&value, output_format, query)
            }
            DeploymentSubCommand::Show { name } => {
                let value = commands::deployment::sub::show::execute(&client, &name).await?;
                output::print_output(&value, output_format, query)
            }
            DeploymentSubCommand::Export { name } => {
                let value = commands::deployment::sub::export::execute(&client, &name).await?;
                output::print_output(&value, output_format, query)
            }
            DeploymentSubCommand::Create { name, location, template_file, template_uri, parameters } => {
                let value = commands::deployment::sub::create::execute(
                    &client, &name, &location,
                    template_file.as_deref(), template_uri.as_deref(),
                    parameters.as_deref(),
                ).await?;
                output::print_output(&value, output_format, query)
            }
            DeploymentSubCommand::Delete { name } => {
                commands::deployment::sub::delete::execute(&client, &name).await
            }
            DeploymentSubCommand::Validate { name, location, template_file, template_uri, parameters } => {
                let deploy_name = name.unwrap_or_else(|| "validation".to_string());
                let value = commands::deployment::sub::validate::execute(
                    &client, &deploy_name, &location,
                    template_file.as_deref(), template_uri.as_deref(),
                    parameters.as_deref(),
                ).await?;
                output::print_output(&value, output_format, query)
            }
            DeploymentSubCommand::WhatIf { name, location, template_file, template_uri, parameters, result_format } => {
                let deploy_name = name.unwrap_or_else(|| "what-if".to_string());
                let value = commands::deployment::sub::what_if::execute(
                    &client, &deploy_name, &location,
                    template_file.as_deref(), template_uri.as_deref(),
                    parameters.as_deref(), Some(&result_format),
                ).await?;
                output::print_output(&value, output_format, query)
            }
            DeploymentSubCommand::Cancel { name } => {
                commands::deployment::sub::cancel::execute(&client, &name).await
            }
            DeploymentSubCommand::Wait { name, created, updated, deleted, exists, interval, timeout } => {
                commands::deployment::sub::wait::execute(&client, &name, created, updated, deleted, exists, interval, timeout).await
            }
        },
        DeploymentCommand::Mg { command } => match command {
            DeploymentMgCommand::List { management_group_id } => {
                let value = commands::deployment::mg::list::execute(&client, &management_group_id).await?;
                output::print_output(&value, output_format, query)
            }
            DeploymentMgCommand::Show { name, management_group_id } => {
                let value = commands::deployment::mg::show::execute(&client, &management_group_id, &name).await?;
                output::print_output(&value, output_format, query)
            }
            DeploymentMgCommand::Export { name, management_group_id } => {
                let value = commands::deployment::mg::export::execute(&client, &management_group_id, &name).await?;
                output::print_output(&value, output_format, query)
            }
            DeploymentMgCommand::Create { name, management_group_id, location, template_file, template_uri, parameters } => {
                let value = commands::deployment::mg::create::execute(
                    &client, &management_group_id, &name, &location,
                    template_file.as_deref(), template_uri.as_deref(),
                    parameters.as_deref(),
                ).await?;
                output::print_output(&value, output_format, query)
            }
            DeploymentMgCommand::Delete { name, management_group_id } => {
                commands::deployment::mg::delete::execute(&client, &management_group_id, &name).await
            }
            DeploymentMgCommand::Validate { name, management_group_id, location, template_file, template_uri, parameters } => {
                let deploy_name = name.unwrap_or_else(|| "validation".to_string());
                let value = commands::deployment::mg::validate::execute(
                    &client, &management_group_id, &deploy_name, &location,
                    template_file.as_deref(), template_uri.as_deref(),
                    parameters.as_deref(),
                ).await?;
                output::print_output(&value, output_format, query)
            }
            DeploymentMgCommand::WhatIf { name, management_group_id, location, template_file, template_uri, parameters, result_format } => {
                let deploy_name = name.unwrap_or_else(|| "what-if".to_string());
                let value = commands::deployment::mg::what_if::execute(
                    &client, &management_group_id, &deploy_name, &location,
                    template_file.as_deref(), template_uri.as_deref(),
                    parameters.as_deref(), Some(&result_format),
                ).await?;
                output::print_output(&value, output_format, query)
            }
            DeploymentMgCommand::Cancel { name, management_group_id } => {
                commands::deployment::mg::cancel::execute(&client, &management_group_id, &name).await
            }
            DeploymentMgCommand::Wait { name, management_group_id, created, updated, deleted, exists, interval, timeout } => {
                commands::deployment::mg::wait::execute(&client, &management_group_id, &name, created, updated, deleted, exists, interval, timeout).await
            }
        },
        DeploymentCommand::Tenant { command } => match command {
            DeploymentTenantCommand::List => {
                let value = commands::deployment::tenant::list::execute(&client).await?;
                output::print_output(&value, output_format, query)
            }
            DeploymentTenantCommand::Show { name } => {
                let value = commands::deployment::tenant::show::execute(&client, &name).await?;
                output::print_output(&value, output_format, query)
            }
            DeploymentTenantCommand::Export { name } => {
                let value = commands::deployment::tenant::export::execute(&client, &name).await?;
                output::print_output(&value, output_format, query)
            }
            DeploymentTenantCommand::Create { name, location, template_file, template_uri, parameters } => {
                let value = commands::deployment::tenant::create::execute(
                    &client, &name, &location,
                    template_file.as_deref(), template_uri.as_deref(),
                    parameters.as_deref(),
                ).await?;
                output::print_output(&value, output_format, query)
            }
            DeploymentTenantCommand::Delete { name } => {
                commands::deployment::tenant::delete::execute(&client, &name).await
            }
            DeploymentTenantCommand::Validate { name, location, template_file, template_uri, parameters } => {
                let deploy_name = name.unwrap_or_else(|| "validation".to_string());
                let value = commands::deployment::tenant::validate::execute(
                    &client, &deploy_name, &location,
                    template_file.as_deref(), template_uri.as_deref(),
                    parameters.as_deref(),
                ).await?;
                output::print_output(&value, output_format, query)
            }
            DeploymentTenantCommand::WhatIf { name, location, template_file, template_uri, parameters, result_format } => {
                let deploy_name = name.unwrap_or_else(|| "what-if".to_string());
                let value = commands::deployment::tenant::what_if::execute(
                    &client, &deploy_name, &location,
                    template_file.as_deref(), template_uri.as_deref(),
                    parameters.as_deref(), Some(&result_format),
                ).await?;
                output::print_output(&value, output_format, query)
            }
            DeploymentTenantCommand::Cancel { name } => {
                commands::deployment::tenant::cancel::execute(&client, &name).await
            }
            DeploymentTenantCommand::Wait { name, created, updated, deleted, exists, interval, timeout } => {
                commands::deployment::tenant::wait::execute(&client, &name, created, updated, deleted, exists, interval, timeout).await
            }
        },
        DeploymentCommand::Operation { command } => match command {
            DeploymentOperationCommand::Group { command } => match command {
                DeploymentOperationGroupCommand::List { name, resource_group } => {
                    let value = commands::deployment::operation::group::list::execute(&client, &resource_group, &name).await?;
                    output::print_output(&value, output_format, query)
                }
                DeploymentOperationGroupCommand::Show { name, resource_group, operation_id } => {
                    let value = commands::deployment::operation::group::show::execute(&client, &resource_group, &name, &operation_id).await?;
                    output::print_output(&value, output_format, query)
                }
            },
            DeploymentOperationCommand::Sub { command } => match command {
                DeploymentOperationSubCommand::List { name } => {
                    let value = commands::deployment::operation::sub::list::execute(&client, &name).await?;
                    output::print_output(&value, output_format, query)
                }
                DeploymentOperationSubCommand::Show { name, operation_id } => {
                    let value = commands::deployment::operation::sub::show::execute(&client, &name, &operation_id).await?;
                    output::print_output(&value, output_format, query)
                }
            },
            DeploymentOperationCommand::Mg { command } => match command {
                DeploymentOperationMgCommand::List { name, management_group_id } => {
                    let value = commands::deployment::operation::mg::list::execute(&client, &management_group_id, &name).await?;
                    output::print_output(&value, output_format, query)
                }
                DeploymentOperationMgCommand::Show { name, management_group_id, operation_id } => {
                    let value = commands::deployment::operation::mg::show::execute(&client, &management_group_id, &name, &operation_id).await?;
                    output::print_output(&value, output_format, query)
                }
            },
            DeploymentOperationCommand::Tenant { command } => match command {
                DeploymentOperationTenantCommand::List { name } => {
                    let value = commands::deployment::operation::tenant::list::execute(&client, &name).await?;
                    output::print_output(&value, output_format, query)
                }
                DeploymentOperationTenantCommand::Show { name, operation_id } => {
                    let value = commands::deployment::operation::tenant::show::execute(&client, &name, &operation_id).await?;
                    output::print_output(&value, output_format, query)
                }
            },
        },
    }
}

async fn handle_bastion(
    cmd: BastionCommand,
    output_format: OutputFormat,
    subscription: Option<String>,
    query: Option<&str>,
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
            output::print_output(&value, output_format, query)
        }
        BastionCommand::Delete {
            name,
            resource_group,
        } => client.delete(&resource_group, &name).await,
        BastionCommand::List { resource_group } => {
            let value = commands::list::execute_with_client(&client, resource_group.as_deref()).await?;
            output::print_output(&value, output_format, query)
        }
        BastionCommand::Show {
            name,
            resource_group,
        } => {
            let value = commands::show::execute_with_client(&client, &resource_group, &name).await?;
            output::print_output(&value, output_format, query)
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
            output::print_output(&value, output_format, query)
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

async fn handle_network_vnet(
    cmd: VnetCommand,
    output_format: OutputFormat,
    subscription: Option<String>,
    query: Option<&str>,
) -> anyhow::Result<()> {
    let mut provider = auth::TokenProvider::load(subscription)?;
    let access_token = provider.get_access_token().await?;
    let subscription_id = provider.get_subscription_id_or_fallback().await?;
    let client = arm_client::ArmClient::new(access_token, subscription_id);

    match cmd {
        VnetCommand::List { resource_group } => {
            let value = commands::network::vnet::list::execute(&client, resource_group.as_deref()).await?;
            output::print_output(&value, output_format, query)
        }
        VnetCommand::Show { name, resource_group } => {
            let value = commands::network::vnet::show::execute(&client, &resource_group, &name).await?;
            output::print_output(&value, output_format, query)
        }
        VnetCommand::Subnet { command } => match command {
            VnetSubnetCommand::List { resource_group, vnet_name } => {
                let value = commands::network::vnet::list_subnets::execute(&client, &resource_group, &vnet_name).await?;
                output::print_output(&value, output_format, query)
            }
            VnetSubnetCommand::Show { name, resource_group, vnet_name } => {
                let value = commands::network::vnet::show_subnet::execute(&client, &resource_group, &vnet_name, &name).await?;
                output::print_output(&value, output_format, query)
            }
        },
        VnetCommand::Peering { command } => match command {
            VnetPeeringCommand::List { resource_group, vnet_name } => {
                let value = commands::network::vnet::list_peerings::execute(&client, &resource_group, &vnet_name).await?;
                output::print_output(&value, output_format, query)
            }
            VnetPeeringCommand::Show { name, resource_group, vnet_name } => {
                let value = commands::network::vnet::show_peering::execute(&client, &resource_group, &vnet_name, &name).await?;
                output::print_output(&value, output_format, query)
            }
        },
    }
}

async fn handle_network_nsg(
    cmd: NsgCommand,
    output_format: OutputFormat,
    subscription: Option<String>,
    query: Option<&str>,
) -> anyhow::Result<()> {
    let mut provider = auth::TokenProvider::load(subscription)?;
    let access_token = provider.get_access_token().await?;
    let subscription_id = provider.get_subscription_id_or_fallback().await?;
    let client = arm_client::ArmClient::new(access_token, subscription_id);

    match cmd {
        NsgCommand::List { resource_group } => {
            let value = commands::network::nsg::list::execute(&client, resource_group.as_deref()).await?;
            output::print_output(&value, output_format, query)
        }
        NsgCommand::Show { name, resource_group } => {
            let value = commands::network::nsg::show::execute(&client, &resource_group, &name).await?;
            output::print_output(&value, output_format, query)
        }
        NsgCommand::Rule { command } => match command {
            NsgRuleCommand::List { resource_group, nsg_name } => {
                let value = commands::network::nsg::list_rules::execute(&client, &resource_group, &nsg_name).await?;
                output::print_output(&value, output_format, query)
            }
            NsgRuleCommand::Show { name, resource_group, nsg_name } => {
                let value = commands::network::nsg::show_rule::execute(&client, &resource_group, &nsg_name, &name).await?;
                output::print_output(&value, output_format, query)
            }
        },
    }
}

async fn handle_network_public_ip(
    cmd: PublicIpCommand,
    output_format: OutputFormat,
    subscription: Option<String>,
    query: Option<&str>,
) -> anyhow::Result<()> {
    let mut provider = auth::TokenProvider::load(subscription)?;
    let access_token = provider.get_access_token().await?;
    let subscription_id = provider.get_subscription_id_or_fallback().await?;
    let client = arm_client::ArmClient::new(access_token, subscription_id);

    match cmd {
        PublicIpCommand::List { resource_group } => {
            let value = commands::network::public_ip::list::execute(&client, resource_group.as_deref()).await?;
            output::print_output(&value, output_format, query)
        }
        PublicIpCommand::Show { name, resource_group } => {
            let value = commands::network::public_ip::show::execute(&client, &resource_group, &name).await?;
            output::print_output(&value, output_format, query)
        }
    }
}

async fn handle_network_nic(
    cmd: NicCommand,
    output_format: OutputFormat,
    subscription: Option<String>,
    query: Option<&str>,
) -> anyhow::Result<()> {
    let mut provider = auth::TokenProvider::load(subscription)?;
    let access_token = provider.get_access_token().await?;
    let subscription_id = provider.get_subscription_id_or_fallback().await?;
    let client = arm_client::ArmClient::new(access_token, subscription_id);

    match cmd {
        NicCommand::List { resource_group } => {
            let value = commands::network::nic::list::execute(&client, resource_group.as_deref()).await?;
            output::print_output(&value, output_format, query)
        }
        NicCommand::Show { name, resource_group } => {
            let value = commands::network::nic::show::execute(&client, &resource_group, &name).await?;
            output::print_output(&value, output_format, query)
        }
        NicCommand::IpConfig { command } => match command {
            NicIpConfigCommand::List { resource_group, nic_name } => {
                let value = commands::network::nic::list_ip_configs::execute(&client, &resource_group, &nic_name).await?;
                output::print_output(&value, output_format, query)
            }
            NicIpConfigCommand::Show { name, resource_group, nic_name } => {
                let value = commands::network::nic::show_ip_config::execute(&client, &resource_group, &nic_name, &name).await?;
                output::print_output(&value, output_format, query)
            }
        },
    }
}

async fn handle_network_private_endpoint(
    cmd: PrivateEndpointCommand,
    output_format: OutputFormat,
    subscription: Option<String>,
    query: Option<&str>,
) -> anyhow::Result<()> {
    let mut provider = auth::TokenProvider::load(subscription)?;
    let access_token = provider.get_access_token().await?;
    let subscription_id = provider.get_subscription_id_or_fallback().await?;
    let client = arm_client::ArmClient::new(access_token, subscription_id);

    match cmd {
        PrivateEndpointCommand::List { resource_group } => {
            let value = commands::network::private_endpoint::list::execute(&client, resource_group.as_deref()).await?;
            output::print_output(&value, output_format, query)
        }
        PrivateEndpointCommand::Show { name, resource_group } => {
            let value = commands::network::private_endpoint::show::execute(&client, &resource_group, &name).await?;
            output::print_output(&value, output_format, query)
        }
    }
}

async fn handle_network_lb(
    cmd: LoadBalancerCommand,
    output_format: OutputFormat,
    subscription: Option<String>,
    query: Option<&str>,
) -> anyhow::Result<()> {
    let mut provider = auth::TokenProvider::load(subscription)?;
    let access_token = provider.get_access_token().await?;
    let subscription_id = provider.get_subscription_id_or_fallback().await?;
    let client = arm_client::ArmClient::new(access_token, subscription_id);

    match cmd {
        LoadBalancerCommand::List { resource_group } => {
            let value = commands::network::lb::list::execute(&client, resource_group.as_deref()).await?;
            output::print_output(&value, output_format, query)
        }
        LoadBalancerCommand::Show { name, resource_group } => {
            let value = commands::network::lb::show::execute(&client, &resource_group, &name).await?;
            output::print_output(&value, output_format, query)
        }
        LoadBalancerCommand::ListMapping { name, resource_group } => {
            let value = commands::network::lb::list_mapping::execute(&client, &resource_group, &name).await?;
            output::print_output(&value, output_format, query)
        }
        LoadBalancerCommand::ListNic { name, resource_group } => {
            let value = commands::network::lb::list_nic::execute(&client, &resource_group, &name).await?;
            output::print_output(&value, output_format, query)
        }
        LoadBalancerCommand::AddressPool { command } => {
            match command {
                LoadBalancerAddressPoolCommand::List { lb_name, resource_group } => {
                    let value = commands::network::lb::address_pool_list::execute(&client, &resource_group, &lb_name).await?;
                    output::print_output(&value, output_format, query)
                }
                LoadBalancerAddressPoolCommand::Show { name, lb_name, resource_group } => {
                    let value = commands::network::lb::address_pool_show::execute(&client, &resource_group, &lb_name, &name).await?;
                    output::print_output(&value, output_format, query)
                }
            }
        }
        LoadBalancerCommand::FrontendIp { command } => {
            match command {
                LoadBalancerFrontendIpCommand::List { lb_name, resource_group } => {
                    let value = commands::network::lb::frontend_ip_list::execute(&client, &resource_group, &lb_name).await?;
                    output::print_output(&value, output_format, query)
                }
                LoadBalancerFrontendIpCommand::Show { name, lb_name, resource_group } => {
                    let value = commands::network::lb::frontend_ip_show::execute(&client, &resource_group, &lb_name, &name).await?;
                    output::print_output(&value, output_format, query)
                }
            }
        }
        LoadBalancerCommand::InboundNatPool { command } => {
            match command {
                LoadBalancerInboundNatPoolCommand::List { lb_name, resource_group } => {
                    let value = commands::network::lb::inbound_nat_pool_list::execute(&client, &resource_group, &lb_name).await?;
                    output::print_output(&value, output_format, query)
                }
                LoadBalancerInboundNatPoolCommand::Show { name, lb_name, resource_group } => {
                    let value = commands::network::lb::inbound_nat_pool_show::execute(&client, &resource_group, &lb_name, &name).await?;
                    output::print_output(&value, output_format, query)
                }
            }
        }
        LoadBalancerCommand::InboundNatRule { command } => {
            match command {
                LoadBalancerInboundNatRuleCommand::List { lb_name, resource_group } => {
                    let value = commands::network::lb::inbound_nat_rule_list::execute(&client, &resource_group, &lb_name).await?;
                    output::print_output(&value, output_format, query)
                }
                LoadBalancerInboundNatRuleCommand::Show { name, lb_name, resource_group } => {
                    let value = commands::network::lb::inbound_nat_rule_show::execute(&client, &resource_group, &lb_name, &name).await?;
                    output::print_output(&value, output_format, query)
                }
            }
        }
        LoadBalancerCommand::OutboundRule { command } => {
            match command {
                LoadBalancerOutboundRuleCommand::List { lb_name, resource_group } => {
                    let value = commands::network::lb::outbound_rule_list::execute(&client, &resource_group, &lb_name).await?;
                    output::print_output(&value, output_format, query)
                }
                LoadBalancerOutboundRuleCommand::Show { name, lb_name, resource_group } => {
                    let value = commands::network::lb::outbound_rule_show::execute(&client, &resource_group, &lb_name, &name).await?;
                    output::print_output(&value, output_format, query)
                }
            }
        }
        LoadBalancerCommand::Probe { command } => {
            match command {
                LoadBalancerProbeCommand::List { lb_name, resource_group } => {
                    let value = commands::network::lb::probe_list::execute(&client, &resource_group, &lb_name).await?;
                    output::print_output(&value, output_format, query)
                }
                LoadBalancerProbeCommand::Show { name, lb_name, resource_group } => {
                    let value = commands::network::lb::probe_show::execute(&client, &resource_group, &lb_name, &name).await?;
                    output::print_output(&value, output_format, query)
                }
            }
        }
        LoadBalancerCommand::Rule { command } => {
            match command {
                LoadBalancerRuleCommand::List { lb_name, resource_group } => {
                    let value = commands::network::lb::rule_list::execute(&client, &resource_group, &lb_name).await?;
                    output::print_output(&value, output_format, query)
                }
                LoadBalancerRuleCommand::Show { name, lb_name, resource_group } => {
                    let value = commands::network::lb::rule_show::execute(&client, &resource_group, &lb_name, &name).await?;
                    output::print_output(&value, output_format, query)
                }
            }
        }
    }
}

async fn handle_network_route_table(
    cmd: RouteTableCommand,
    output_format: OutputFormat,
    subscription: Option<String>,
    query: Option<&str>,
) -> anyhow::Result<()> {
    let mut provider = auth::TokenProvider::load(subscription)?;
    let access_token = provider.get_access_token().await?;
    let subscription_id = provider.get_subscription_id_or_fallback().await?;
    let client = arm_client::ArmClient::new(access_token, subscription_id);

    match cmd {
        RouteTableCommand::List { resource_group } => {
            let value = commands::network::route_table::list::execute(&client, resource_group.as_deref()).await?;
            output::print_output(&value, output_format, query)
        }
        RouteTableCommand::Show { name, resource_group } => {
            let value = commands::network::route_table::show::execute(&client, &resource_group, &name).await?;
            output::print_output(&value, output_format, query)
        }
        RouteTableCommand::Route { command } => {
            match command {
                RouteTableRouteCommand::List { table_name, resource_group } => {
                    let value = commands::network::route_table::route_list::execute(&client, &resource_group, &table_name).await?;
                    output::print_output(&value, output_format, query)
                }
                RouteTableRouteCommand::Show { name, table_name, resource_group } => {
                    let value = commands::network::route_table::route_show::execute(&client, &resource_group, &table_name, &name).await?;
                    output::print_output(&value, output_format, query)
                }
            }
        }
    }
}

async fn handle_network_dns(
    cmd: DnsCommand,
    output_format: OutputFormat,
    subscription: Option<String>,
    query: Option<&str>,
) -> anyhow::Result<()> {
    let mut provider = auth::TokenProvider::load(subscription)?;
    let access_token = provider.get_access_token().await?;
    let subscription_id = provider.get_subscription_id_or_fallback().await?;
    let client = arm_client::ArmClient::new(access_token, subscription_id);

    match cmd {
        DnsCommand::ListZone { resource_group } => {
            let value = commands::network::dns::list_zone::execute(&client, resource_group.as_deref()).await?;
            output::print_output(&value, output_format, query)
        }
        DnsCommand::ShowZone { name, resource_group } => {
            let value = commands::network::dns::show_zone::execute(&client, &resource_group, &name).await?;
            output::print_output(&value, output_format, query)
        }
        DnsCommand::RecordSet { command } => {
            match command {
                DnsRecordSetCommand::List { zone_name, resource_group, record_type } => {
                    let value = commands::network::dns::record_set_list::execute(&client, &resource_group, &zone_name, &record_type).await?;
                    output::print_output(&value, output_format, query)
                }
                DnsRecordSetCommand::Show { name, zone_name, resource_group, record_type } => {
                    let value = commands::network::dns::record_set_show::execute(&client, &resource_group, &zone_name, &name, &record_type).await?;
                    output::print_output(&value, output_format, query)
                }
            }
        }
    }
}




async fn handle_network_watcher(
    cmd: WatcherCommand,
    output_format: OutputFormat,
    subscription: Option<String>,
    query: Option<&str>,
) -> anyhow::Result<()> {
    let mut provider = auth::TokenProvider::load(subscription)?;
    let access_token = provider.get_access_token().await?;
    let subscription_id = provider.get_subscription_id_or_fallback().await?;
    let client = arm_client::ArmClient::new(access_token, subscription_id);

    match cmd {
        WatcherCommand::List => {
            let value = commands::network::watcher::list::execute(&client).await?;
            output::print_output(&value, output_format, query)
        }
        WatcherCommand::Show { name, resource_group } => {
            let value = commands::network::watcher::show::execute(&client, &resource_group, &name).await?;
            output::print_output(&value, output_format, query)
        }
        WatcherCommand::ConnectionMonitor { command } => {
            match command {
                WatcherConnectionMonitorCommand::List { watcher_name, resource_group } => {
                    let value = commands::network::watcher::connection_monitor_list::execute(&client, &resource_group, &watcher_name).await?;
                    output::print_output(&value, output_format, query)
                }
                WatcherConnectionMonitorCommand::Show { name, watcher_name, resource_group } => {
                    let value = commands::network::watcher::connection_monitor_show::execute(&client, &resource_group, &watcher_name, &name).await?;
                    output::print_output(&value, output_format, query)
                }
            }
        }
        WatcherCommand::FlowLog { command } => {
            match command {
                WatcherFlowLogCommand::List { watcher_name, resource_group } => {
                    let value = commands::network::watcher::flow_log_list::execute(&client, &resource_group, &watcher_name).await?;
                    output::print_output(&value, output_format, query)
                }
                WatcherFlowLogCommand::Show { name, watcher_name, resource_group } => {
                    let value = commands::network::watcher::flow_log_show::execute(&client, &resource_group, &watcher_name, &name).await?;
                    output::print_output(&value, output_format, query)
                }
            }
        }
        WatcherCommand::PacketCapture { command } => {
            match command {
                WatcherPacketCaptureCommand::List { watcher_name, resource_group } => {
                    let value = commands::network::watcher::packet_capture_list::execute(&client, &resource_group, &watcher_name).await?;
                    output::print_output(&value, output_format, query)
                }
                WatcherPacketCaptureCommand::Show { name, watcher_name, resource_group } => {
                    let value = commands::network::watcher::packet_capture_show::execute(&client, &resource_group, &watcher_name, &name).await?;
                    output::print_output(&value, output_format, query)
                }
            }
        }
    }
}
