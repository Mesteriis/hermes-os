//! Kernel-owned admission for the narrow browser Gateway foundation.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use hermes_gateway_runtime::{
    BrowserBootstrapRouter, BrowserPairingRouter, ClientRpcRouteErrorV1, ClientRpcRouteHandler,
    ClientRpcRouteV1, ClientRpcRouter, GatewayApplicationRouter, GatewayHttp3ListenerV1,
    GatewayLanDevelopmentListenerV1, GatewayLoopbackTlsListenerV1, GatewayTechnicalRouter,
    GatewayTlsListenerV1, InMemoryBrowserRealtimeSource, PairedRemoteProfileV1,
    SharedBrowserPairingManager,
};
use hermes_gateway_session::{
    BrowserGatewaySessionService, BrowserPairingChallengeV1, BrowserPairingManager,
    BrowserWebauthnVerifier, OwnerPairingApprovalV1,
};
use hermes_kernel_control_store_sqlite::SqliteControlStore;
use hermes_runtime_protocol::v1::{
    ContractReferenceV1, DistributionArtifactKindV1, DistributionManifestArtifactV1,
    ModuleClientRequestV1, ModuleClientResponseV1,
};
use prost::Message;
use tokio::sync::watch;

use crate::identity::browser_gateway::ControlStoreBrowserAuthority;
use crate::modules::capability::router::{
    ManagedCapabilityRouteRequest, route_managed_client_request,
};
use crate::platform::macos::native_launch;
use crate::runtime::lifecycle::supervisor::ManagedRuntimeSupervisor;

const SHUTDOWN_POLL: Duration = Duration::from_millis(25);
const BROWSER_BOOTSTRAP_ARTIFACT_ID: &str = "browser.bootstrap";
const MACOS_KERNEL_TARGET: &str = "aarch64-apple-darwin";
const MODULE_CLIENT_PROTOCOL_MAJOR: u32 = 1;

#[path = "gateway/tls.rs"]
mod tls;

/// Explicit operator-owned parameters for a browser Gateway. TLS material is
/// absent only in the private-LAN HTTP developer profile.
#[derive(Clone)]
pub(crate) struct BrowserGatewayConfigurationV1 {
    listen_address: SocketAddr,
    exact_https_origin: String,
    rp_id: String,
    certificate_der_path: Option<PathBuf>,
    private_key_der_path: Option<PathBuf>,
    exposure: BrowserGatewayExposureV1,
}

#[derive(Clone, Copy)]
enum BrowserGatewayExposureV1 {
    LocalEmbedded,
    PairedRemote,
    LanDevelopment,
}

/// Kernel-owned bridge between a private owner-control approval and the
/// Gateway's public registration adapter. The opaque state never reaches the
/// browser until the approved ID is used at the exact configured origin.
pub(crate) struct BrowserPairingAdmissionV1 {
    pairings: SharedBrowserPairingManager,
    authority: ControlStoreBrowserAuthority,
    verifier: BrowserWebauthnVerifier,
}

impl BrowserPairingAdmissionV1 {
    pub(crate) fn new(
        store: Arc<SqliteControlStore>,
        supervisor: ManagedRuntimeSupervisor,
        configuration: &BrowserGatewayConfigurationV1,
    ) -> Result<Self, String> {
        Ok(Self {
            pairings: Arc::new(Mutex::new(BrowserPairingManager::default())),
            authority: ControlStoreBrowserAuthority::new(store, supervisor),
            verifier: BrowserWebauthnVerifier::new(
                &configuration.rp_id,
                &configuration.exact_https_origin,
            )
            .map_err(|_| "browser Gateway origin or RP ID is invalid".to_owned())?,
        })
    }

    pub(crate) fn begin(
        &self,
        owner_id: &str,
        authorizing_device_id: &str,
        now_unix_millis: u64,
    ) -> Result<BrowserPairingChallengeV1, String> {
        let approval = OwnerPairingApprovalV1::new(owner_id, authorizing_device_id)?;
        self.pairings
            .lock()
            .map_err(|_| "browser pairing is unavailable".to_owned())?
            .begin_webauthn(&self.authority, &self.verifier, approval, now_unix_millis)
            .map(|ceremony| ceremony.pairing().clone())
    }

    fn router(
        &self,
        configuration: &BrowserGatewayConfigurationV1,
    ) -> Result<BrowserPairingRouter<ControlStoreBrowserAuthority>, String> {
        let verifier =
            BrowserWebauthnVerifier::new(&configuration.rp_id, &configuration.exact_https_origin)
                .map_err(|_| "browser Gateway origin or RP ID is invalid".to_owned())?;
        Ok(BrowserPairingRouter::new(
            Arc::clone(&self.pairings),
            self.authority.clone(),
            verifier,
            configuration.exact_https_origin.clone(),
        ))
    }
}

impl BrowserGatewayConfigurationV1 {
    pub(crate) fn new(
        listen_address: SocketAddr,
        exact_https_origin: String,
        rp_id: String,
        certificate_der_path: PathBuf,
        private_key_der_path: PathBuf,
    ) -> Result<Self, String> {
        (!exact_https_origin.is_empty() && !rp_id.is_empty())
            .then_some(())
            .ok_or_else(|| "browser Gateway origin and RP ID are required".to_owned())?;
        BrowserWebauthnVerifier::new(&rp_id, &exact_https_origin)
            .map_err(|_| "browser Gateway origin or RP ID is invalid".to_owned())?;
        listen_address
            .ip()
            .is_loopback()
            .then_some(())
            .ok_or_else(|| "browser Gateway listener must bind loopback only".to_owned())?;
        Ok(Self {
            listen_address,
            exact_https_origin,
            rp_id,
            certificate_der_path: Some(certificate_der_path),
            private_key_der_path: Some(private_key_der_path),
            exposure: BrowserGatewayExposureV1::LocalEmbedded,
        })
    }

    pub(crate) fn new_paired_remote(
        listen_address: SocketAddr,
        exact_https_origin: String,
        rp_id: String,
        certificate_der_path: PathBuf,
        private_key_der_path: PathBuf,
    ) -> Result<Self, String> {
        (!exact_https_origin.is_empty() && !rp_id.is_empty())
            .then_some(())
            .ok_or_else(|| "browser Gateway origin and RP ID are required".to_owned())?;
        BrowserWebauthnVerifier::new(&rp_id, &exact_https_origin)
            .map_err(|_| "browser Gateway origin or RP ID is invalid".to_owned())?;
        Ok(Self {
            listen_address,
            exact_https_origin,
            rp_id,
            certificate_der_path: Some(certificate_der_path),
            private_key_der_path: Some(private_key_der_path),
            exposure: BrowserGatewayExposureV1::PairedRemote,
        })
    }

    pub(crate) fn new_lan_development(
        listen_address: SocketAddr,
        exact_https_origin: String,
        rp_id: String,
    ) -> Result<Self, String> {
        require_private_lan_address(listen_address)?;
        let origin_address = exact_https_origin
            .strip_prefix("http://")
            .and_then(|authority| authority.parse::<SocketAddr>().ok())
            .filter(|address| *address == listen_address)
            .ok_or_else(|| {
                "developer mode requires an HTTP origin equal to the private LAN listener"
                    .to_owned()
            })?;
        (rp_id == origin_address.ip().to_string())
            .then_some(())
            .ok_or_else(|| "developer mode RP ID must equal the private LAN IP".to_owned())?;
        Ok(Self {
            listen_address,
            exact_https_origin,
            rp_id,
            certificate_der_path: None,
            private_key_der_path: None,
            exposure: BrowserGatewayExposureV1::LanDevelopment,
        })
    }

    pub(crate) fn is_lan_development(&self) -> bool {
        matches!(self.exposure, BrowserGatewayExposureV1::LanDevelopment)
    }
}

fn require_private_lan_address(address: SocketAddr) -> Result<(), String> {
    let private = match address.ip() {
        std::net::IpAddr::V4(ip) => ip.is_private() || ip.is_link_local(),
        std::net::IpAddr::V6(ip) => {
            let first = ip.segments()[0];
            (first & 0xfe00) == 0xfc00 || (first & 0xffc0) == 0xfe80
        }
    };
    private
        .then_some(())
        .ok_or_else(|| "developer mode listener must bind a private LAN address".to_owned())
}

pub(crate) fn serve(
    store: Arc<SqliteControlStore>,
    supervisor: ManagedRuntimeSupervisor,
    configuration: BrowserGatewayConfigurationV1,
    pairing: Option<Arc<BrowserPairingAdmissionV1>>,
    shutdown_requested: Arc<AtomicBool>,
) -> Result<(), String> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .enable_time()
        .build()
        .map_err(|_| "browser Gateway runtime is unavailable".to_owned())?;
    runtime.block_on(serve_async(
        store,
        supervisor,
        configuration,
        pairing,
        shutdown_requested,
    ))
}

async fn serve_async(
    store: Arc<SqliteControlStore>,
    supervisor: ManagedRuntimeSupervisor,
    configuration: BrowserGatewayConfigurationV1,
    pairing: Option<Arc<BrowserPairingAdmissionV1>>,
    shutdown_requested: Arc<AtomicBool>,
) -> Result<(), String> {
    let service = gateway_service(store, supervisor, &configuration, pairing)?;
    let (shutdown, receiver) = watch::channel(false);
    let watcher = shutdown_watcher(shutdown_requested, shutdown);
    let result = serve_configured_listener(configuration, service, receiver).await;
    watcher.abort();
    result
}

type BrowserGatewayRouter =
    GatewayApplicationRouter<ControlStoreBrowserAuthority, InMemoryBrowserRealtimeSource>;

pub(crate) fn gateway_service(
    store: Arc<SqliteControlStore>,
    supervisor: ManagedRuntimeSupervisor,
    configuration: &BrowserGatewayConfigurationV1,
    pairing: Option<Arc<BrowserPairingAdmissionV1>>,
) -> Result<BrowserGatewayRouter, String> {
    let authority = ControlStoreBrowserAuthority::new(Arc::clone(&store), supervisor.clone());
    let verifier =
        BrowserWebauthnVerifier::new(&configuration.rp_id, &configuration.exact_https_origin)
            .map_err(|_| "browser Gateway origin or RP ID is invalid".to_owned())?;
    let session = Arc::new(
        BrowserGatewaySessionService::new(
            authority,
            verifier,
            configuration.exact_https_origin.clone(),
        )
        .map_err(|_| "browser Gateway session service is unavailable".to_owned())?,
    );
    let realtime = InMemoryBrowserRealtimeSource::new(1_024)
        .map_err(|_| "browser Gateway realtime source is unavailable".to_owned())?;
    let request_id_sequence = Arc::new(AtomicU64::new(0));
    let client_rpc_routes = store
        .approved_module_client_rpc_routes()
        .map_err(|_| "owner ClientRpc route records are unavailable".to_owned())?;
    let client_rpc_handler: ClientRpcRouteHandler = {
        let store = Arc::clone(&store);
        let relay = supervisor.relay_port();
        let request_id_sequence = Arc::clone(&request_id_sequence);
        Arc::new(
            move |route: &ClientRpcRouteV1, _logical_owner_id: &str, request_payload: &[u8]| {
                // A browser session is authorized for the logical human owner. A
                // route owner is the admitted module/domain namespace, such as
                // `communications`; these identifiers intentionally never match.
                // Session authorization is completed before this handler runs.
                let snapshot = store
                    .module_grant_snapshot(route.registration_id())
                    .map_err(|_| ClientRpcRouteErrorV1::Internal)?
                    .ok_or(ClientRpcRouteErrorV1::NotFound)?;
                if snapshot.registration().owner_id() != route.owner() {
                    return Err(ClientRpcRouteErrorV1::NotFound);
                }
                let grants = snapshot
                    .effective_grants()
                    .ok_or(ClientRpcRouteErrorV1::NotFound)?;
                if grants
                    .capability_ids()
                    .binary_search_by(|candidate| candidate.as_str().cmp(route.capability_id()))
                    .is_err()
                {
                    return Err(ClientRpcRouteErrorV1::NotFound);
                }
                let launch = store
                    .effective_managed_launch_record(route.registration_id())
                    .map_err(|_| ClientRpcRouteErrorV1::Internal)?
                    .ok_or(ClientRpcRouteErrorV1::Unavailable)?;
                let request_id = request_id_sequence.fetch_add(1, Ordering::Relaxed).max(1);
                let request = encode_owner_client_rpc_module_request(
                    snapshot.registration().module_id(),
                    route,
                    request_id,
                    request_payload,
                )
                .map_err(|_| ClientRpcRouteErrorV1::InvalidArgument)?;
                let route = ManagedCapabilityRouteRequest::new(
                    snapshot.registration().registration_id(),
                    launch.runtime_instance_id(),
                    launch.runtime_generation(),
                    grants.grant_epoch(),
                    route.capability_id(),
                    &request,
                );
                let response_bytes = route_managed_client_request(&*store, &relay, &route)
                    .map_err(map_managed_client_rpc_route_error)?;
                let response = ModuleClientResponseV1::decode(response_bytes.as_slice())
                    .map_err(|_| ClientRpcRouteErrorV1::Internal)?;
                if !response.error_code.is_empty() {
                    return Err(ClientRpcRouteErrorV1::Internal);
                }
                Ok(response.response_payload)
            },
        )
    };
    let client_rpc_routes = client_rpc_routes
        .into_iter()
        .map(|route| {
            ClientRpcRouter::new(
                Arc::clone(&session),
                ClientRpcRouteV1::new(
                    route.registration_id(),
                    route.capability_id(),
                    route.owner(),
                    route.contract_name(),
                    hermes_gateway_runtime::ClientRpcContractVersionV1 {
                        major: route.contract_major(),
                        revision: route.contract_revision(),
                    },
                    *route.contract_schema_sha256(),
                    route.path(),
                ),
                Arc::clone(&client_rpc_handler),
            )
        })
        .collect();
    let mut service = GatewayApplicationRouter::new(true, Arc::clone(&session), realtime)
        .with_client_rpc_routes(client_rpc_routes)
        .map_err(str::to_owned)?;
    if let Some(pairing) = pairing {
        service = service.with_browser_pairing(pairing.router(configuration)?);
    }
    if let Some(bootstrap) = load_signed_browser_bootstrap()? {
        service = service.with_browser_bootstrap(bootstrap);
    }
    Ok(service)
}

fn shutdown_watcher(
    shutdown_requested: Arc<AtomicBool>,
    shutdown: watch::Sender<bool>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        while !shutdown_requested.load(Ordering::Acquire) {
            tokio::time::sleep(SHUTDOWN_POLL).await;
        }
        let _ = shutdown.send(true);
    })
}

async fn serve_configured_listener(
    configuration: BrowserGatewayConfigurationV1,
    service: BrowserGatewayRouter,
    receiver: watch::Receiver<bool>,
) -> Result<(), String> {
    match configuration.exposure {
        BrowserGatewayExposureV1::LocalEmbedded => {
            let listener = GatewayLoopbackTlsListenerV1::bind(
                configuration.listen_address,
                tls::acceptor(&configuration, None)?,
            )
            .await?;
            println!("browser_gateway_listener={}", listener.local_address()?);
            listener.serve_until_shutdown(service, receiver).await
        }
        BrowserGatewayExposureV1::PairedRemote => {
            let profile = PairedRemoteProfileV1::new(true, false).map_err(str::to_owned)?;
            let http2 = GatewayTlsListenerV1::bind(
                configuration.listen_address,
                profile,
                tls::acceptor(&configuration, Some(b"h2"))?,
            )
            .await?;
            let http3 = GatewayHttp3ListenerV1::bind(
                configuration.listen_address,
                profile,
                tls::http3_server_config(&configuration)?,
            )?;
            println!("browser_gateway_listener={}", http2.local_address()?);
            println!("browser_gateway_http3_listener={}", http3.local_address()?);
            let http2 = http2.serve_until_shutdown(service.clone(), receiver.clone());
            let http3 = http3.serve_until_shutdown(service, receiver);
            tokio::try_join!(http2, http3).map(|_| ())
        }
        BrowserGatewayExposureV1::LanDevelopment => {
            let listener =
                GatewayLanDevelopmentListenerV1::bind(configuration.listen_address).await?;
            println!("developer_mode=enabled");
            println!("developer_mode_authentication=owner_apis_unavailable");
            println!("developer_mode_logging=verbose_sanitized_console");
            println!("developer_mode_ingress=private_lan_http_only");
            println!("developer_mode_egress=unrestricted");
            println!("browser_gateway_listener={}", listener.local_address()?);
            listener
                .serve_until_shutdown(GatewayTechnicalRouter::new(true), receiver)
                .await
        }
    }
}

fn load_signed_browser_bootstrap() -> Result<Option<BrowserBootstrapRouter>, String> {
    let executable =
        std::env::current_exe().map_err(|_| "Kernel executable path is unavailable".to_owned())?;
    let bundle =
        match native_launch::verify_selected_installed_bundle(&executable, MACOS_KERNEL_TARGET) {
            Ok(bundle) => bundle,
            Err(error) if error == "Kernel executable is not inside a macOS app bundle" => {
                return Ok(None);
            }
            Err(_) => return Err("signed browser bootstrap release verification failed".to_owned()),
        };
    let manifest = required_browser_bootstrap_manifest(&bundle.manifest().artifacts)?;
    if manifest.artifact_kind != DistributionArtifactKindV1::BrowserBootstrapBundle as i32
        || !manifest.required
    {
        return Err("signed browser bootstrap manifest artifact is invalid".to_owned());
    }
    let artifact = bundle
        .artifacts()
        .iter()
        .find(|artifact| artifact.artifact_id() == BROWSER_BOOTSTRAP_ARTIFACT_ID)
        .ok_or_else(|| "signed browser bootstrap artifact is unavailable".to_owned())?;
    let assets = bundle
        .manifest()
        .artifacts
        .iter()
        .filter(|candidate| {
            candidate.artifact_kind == DistributionArtifactKindV1::BrowserClientAsset as i32
        })
        .map(|candidate| {
            let path = browser_asset_url(candidate)?;
            let bytes = bundle
                .artifacts()
                .iter()
                .find(|artifact| artifact.artifact_id() == candidate.artifact_id)
                .ok_or_else(|| "signed browser asset is unavailable".to_owned())?
                .read_verified_bytes()?;
            Ok((path, bytes))
        })
        .collect::<Result<Vec<_>, String>>()?;
    BrowserBootstrapRouter::new(artifact.read_verified_bytes()?)?
        .with_assets(assets)
        .map(Some)
}

fn browser_asset_url(artifact: &DistributionManifestArtifactV1) -> Result<String, String> {
    const PREFIX: &str = "browser/assets/";
    let name = artifact
        .relative_path
        .strip_prefix(PREFIX)
        .filter(|name| {
            !name.is_empty()
                && name
                    .split('/')
                    .all(|part| !part.is_empty() && part != "." && part != "..")
        })
        .ok_or_else(|| "signed browser asset manifest path is invalid".to_owned())?;
    Ok(format!("/assets/{name}"))
}

pub(crate) fn required_browser_bootstrap_manifest(
    artifacts: &[DistributionManifestArtifactV1],
) -> Result<&DistributionManifestArtifactV1, String> {
    artifacts
        .iter()
        .find(|artifact| artifact.artifact_id == BROWSER_BOOTSTRAP_ARTIFACT_ID)
        .ok_or_else(|| "signed browser bootstrap artifact is required".to_owned())
}

fn encode_owner_client_rpc_module_request(
    module_id: &str,
    route: &ClientRpcRouteV1,
    request_id: u64,
    request_payload: &[u8],
) -> Result<Vec<u8>, ()> {
    if module_id.is_empty() || request_id == 0 || request_payload.is_empty() {
        return Err(());
    }
    Ok(ModuleClientRequestV1 {
        protocol_major: MODULE_CLIENT_PROTOCOL_MAJOR,
        module_id: module_id.to_owned(),
        owner_id: route.owner().to_owned(),
        contract: Some(ContractReferenceV1 {
            owner: route.owner().to_owned(),
            name: route.contract_name().to_owned(),
            major: route.contract_major(),
            revision: route.contract_revision(),
            schema_sha256: route.contract_schema_sha256().to_vec(),
        }),
        request_id,
        request_payload: request_payload.to_vec(),
    }
    .encode_to_vec())
}

fn map_managed_client_rpc_route_error(error: String) -> ClientRpcRouteErrorV1 {
    match error.as_str() {
        "managed runtime is unavailable" => ClientRpcRouteErrorV1::Unavailable,
        "managed runtime fence is stale" => ClientRpcRouteErrorV1::Unavailable,
        "module registration is not approved" => ClientRpcRouteErrorV1::NotFound,
        "capability is not granted to this registration" => ClientRpcRouteErrorV1::NotFound,
        _ => ClientRpcRouteErrorV1::Internal,
    }
}
