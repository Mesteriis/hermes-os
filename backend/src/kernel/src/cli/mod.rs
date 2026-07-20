//! Production Kernel command-line contract.

use std::net::SocketAddr;
use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

use crate::platform::gateway::BrowserGatewayConfigurationV1;

#[derive(Parser)]
#[command(name = "hermes-kernel")]
pub(crate) struct Cli {
    #[arg(long)]
    pub(crate) data_dir: Option<PathBuf>,
    #[command(subcommand)]
    pub(crate) command: Command,
}

#[derive(Subcommand)]
pub(crate) enum Command {
    Status,
    Serve {
        #[command(flatten)]
        browser_gateway: BrowserGatewayCli,
    },
    DeviceKeyGenerate,
    InitialOwnerEnroll {
        #[arg(long)]
        owner_id: String,
        #[arg(long)]
        device_id: String,
    },
    ServerBootstrapPairing {
        #[arg(long)]
        listen_address: SocketAddr,
        #[arg(long, default_value_t = 300)]
        ttl_seconds: u64,
    },
    BrowserPairing {
        #[command(subcommand)]
        operation: BrowserPairingCommand,
    },
    ControlStore {
        #[command(subcommand)]
        operation: OfflineControlStoreCommand,
    },
}

/// Browser device enrollment is created only through the owner-private server
/// CLI. The browser never obtains a registration ceremony by itself.
#[derive(Subcommand)]
pub(crate) enum BrowserPairingCommand {
    Create,
}

/// Browser Gateway admission is intentionally opt-in. A running Kernel never
/// chooses an origin, RP ID or TLS identity on behalf of its operator.
#[derive(Args)]
pub(crate) struct BrowserGatewayCli {
    #[arg(long)]
    browser_gateway_listen_address: Option<SocketAddr>,
    #[arg(long)]
    browser_gateway_origin: Option<String>,
    #[arg(long)]
    browser_gateway_rp_id: Option<String>,
    #[arg(long)]
    browser_gateway_certificate_der: Option<PathBuf>,
    #[arg(long)]
    browser_gateway_private_key_der: Option<PathBuf>,
    /// Exposes the explicitly configured Gateway as a paired-remote HTTPS
    /// listener with HTTP/2 and HTTP/3. Without this flag the listener stays
    /// loopback-only.
    #[arg(long)]
    browser_gateway_paired_remote: bool,
    /// Enables process-local private-LAN HTTP diagnostics. It never grants owner access.
    #[arg(long)]
    pub(crate) dangerous_lan_development: bool,
}

impl BrowserGatewayCli {
    pub(crate) fn into_configuration(
        self,
    ) -> Result<Option<BrowserGatewayConfigurationV1>, String> {
        match (
            self.browser_gateway_listen_address,
            self.browser_gateway_origin,
            self.browser_gateway_rp_id,
            self.browser_gateway_certificate_der,
            self.browser_gateway_private_key_der,
            self.browser_gateway_paired_remote,
            self.dangerous_lan_development,
        ) {
            (None, None, None, None, None, false, false) => Ok(None),
            (Some(address), Some(origin), Some(rp_id), None, None, false, true) =>
            {
                BrowserGatewayConfigurationV1::new_lan_development(address, origin, rp_id)
                    .map(Some)
            }
            (Some(address), Some(origin), Some(rp_id), Some(certificate), Some(private_key), paired_remote, dangerous_lan_development) => {
                let configuration = if dangerous_lan_development {
                    return Err("developer mode uses private-LAN HTTP and does not accept TLS or paired-remote inputs".to_owned());
                } else if paired_remote {
                    BrowserGatewayConfigurationV1::new_paired_remote(
                        address,
                        origin,
                        rp_id,
                        certificate,
                        private_key,
                    )
                } else {
                    BrowserGatewayConfigurationV1::new(
                        address,
                        origin,
                        rp_id,
                        certificate,
                        private_key,
                    )
                };
                configuration.map(Some)
            }
            _ => Err(
                "browser Gateway listener, origin, RP ID, certificate DER and private-key DER must be specified together; paired remote is explicit"
                    .to_owned(),
            ),
        }
    }
}

#[derive(Subcommand)]
pub(crate) enum OfflineControlStoreCommand {
    Restore {
        #[arg(long)]
        source: PathBuf,
    },
    Reset,
}
