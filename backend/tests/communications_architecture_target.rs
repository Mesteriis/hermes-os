use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("backend has repository parent")
        .to_path_buf()
}

#[test]
fn channel_providers_are_not_product_domains_or_user_routes() {
    let root = repo_root();

    let backend_domains_dir = root.join("backend/src/domains");
    let frontend_domains_dir = root.join("frontend/src/domains");
    assert!(
        !backend_domains_dir.join("mail").exists(),
        "mail must not remain a backend domain"
    );
    assert!(
        root.join("backend/src/domains/communications").is_dir(),
        "communications must be the backend communication domain"
    );
    assert!(
        !frontend_domains_dir.join("telegram").exists(),
        "Telegram must not remain a frontend product domain"
    );
    assert!(
        !frontend_domains_dir.join("whatsapp").exists(),
        "WhatsApp must not remain a frontend product domain"
    );

    let domains_mod = read(root.join("backend/src/domains/mod.rs"));
    assert!(
        !domains_mod.contains("mod mail"),
        "backend domains mod must not export mail"
    );
    assert!(
        domains_mod.contains("mod communications"),
        "backend domains mod must export communications"
    );
    for legacy_mail_runtime_mod in [
        "pub mod accounts;",
        "pub mod background_sync;",
        "pub mod sync;",
        "pub mod rfc822;",
        "pub mod send;",
        "pub mod imap_write;",
    ] {
        assert!(
            !domains_mod.contains(legacy_mail_runtime_mod),
            "communications domain must not own mail runtime module {legacy_mail_runtime_mod}"
        );
    }
    let mail_integrations_mod = read(root.join("backend/src/integrations/mail/mod.rs"));
    for integration_mail_mod in [
        "pub mod accounts;",
        "pub mod sync;",
        "pub mod rfc822;",
        "pub mod send;",
        "pub mod imap_write;",
    ] {
        assert!(
            mail_integrations_mod.contains(integration_mail_mod),
            "mail integrations module must export {integration_mail_mod}"
        );
    }
    let workflows_mod = read(root.join("backend/src/workflows/mod.rs"));
    assert!(
        workflows_mod.contains("pub mod mail_background_sync;"),
        "mail background sync must be a workflow process manager, not an integration module"
    );

    let router_sources = read_all_sources(root.join("backend/src/app/router"));
    let legacy_mail_domain_import = format!("domains::{}", "mail");
    assert!(
        !router_sources.contains(&legacy_mail_domain_import),
        "router code must not import the old mail domain"
    );
    for legacy_prefix in [
        format!("\"/api/v1/{}", "telegram"),
        format!("\"/api/v1/{}", "whatsapp"),
        format!("\"/api/v1/{}", "email-accounts"),
    ] {
        assert!(
            !router_sources.contains(&legacy_prefix),
            "legacy user-facing provider route prefix remains: {legacy_prefix}"
        );
    }
    for integration_prefix in [
        "\"/api/v1/integrations/telegram",
        "\"/api/v1/integrations/whatsapp",
        "\"/api/v1/integrations/mail",
    ] {
        assert!(
            router_sources.contains(integration_prefix),
            "provider runtime/setup routes must live under integrations: {integration_prefix}"
        );
    }
    assert!(
        !router_sources.contains("\"/api/v1/integrations/whatsapp/messages\""),
        "WhatsApp message business reads must use provider-neutral Communications routes"
    );
    for forbidden_communication_prefix in ["mail", "telegram", "whatsapp"]
        .map(|provider| format!("\"/api/v1/communications/{provider}"))
    {
        assert!(
            !router_sources.contains(&forbidden_communication_prefix),
            "provider-specific communication route prefix remains: {forbidden_communication_prefix}"
        );
    }
    for removed_provider_communication_prefix in [
        "\"/api/v1/communications/provider-conversations",
        "\"/api/v1/communications/provider-messages",
        "\"/api/v1/communications/provider-web-messages",
    ] {
        assert!(
            !router_sources.contains(removed_provider_communication_prefix),
            "removed provider-shaped communication route still exists: {removed_provider_communication_prefix}"
        );
    }
    assert!(
        router_sources.contains("\"/api/v1/communications/")
            && !router_sources.contains(&format!("\"/api/v1/communications/{}/accounts", "mail")),
        "communications router must keep product routes under /api/v1/communications without resurrecting mail runtime setup paths"
    );
    for provider_neutral_communication_prefix in [
        "\"/api/v1/communications/messages",
        "\"/api/v1/communications/search",
    ] {
        assert!(
            router_sources.contains(provider_neutral_communication_prefix),
            "communications router must expose provider-neutral communication routes: {provider_neutral_communication_prefix}"
        );
    }

    let telegram_chats_handler =
        read(root.join("backend/src/app/provider_runtime_handlers/telegram/chats.rs"));
    let telegram_search_handler =
        read(root.join("backend/src/app/provider_runtime_handlers/telegram/search.rs"));
    assert!(
        telegram_chats_handler.contains("query.channel_kind.as_deref()")
            && telegram_search_handler.contains("query.channel_kind.as_deref()"),
        "provider-neutral conversation routes must honor channel_kind filtering for WhatsApp/Telegram conversation reads"
    );
    assert!(
        telegram_chats_handler.contains("includes_whatsapp_channel_kind")
            && telegram_chats_handler.contains("includes_telegram_channel_kind")
            && telegram_search_handler.contains("includes_whatsapp_channel_kind")
            && telegram_search_handler.contains("includes_telegram_channel_kind"),
        "provider-neutral conversation routes must not mix Telegram runtime rows into WhatsApp-filtered reads"
    );
    assert!(
        telegram_search_handler.contains("query.channel_kind.as_deref()")
            && telegram_search_handler.contains("search_channel_kinds("),
        "provider-neutral message/media search routes must honor channel_kind filtering for WhatsApp/Telegram reads"
    );

    let frontend_app_sources = read_all_sources(root.join("frontend/src/app"));
    assert!(
        !frontend_app_sources.contains("'/telegram'")
            && !frontend_app_sources.contains("\"/telegram\""),
        "Telegram must not remain a top-level frontend route"
    );
    assert!(
        !frontend_app_sources.contains("'/whatsapp'")
            && !frontend_app_sources.contains("\"/whatsapp\""),
        "WhatsApp must not remain a top-level frontend route"
    );

    let frontend_communications_domain =
        read_all_sources(root.join("frontend/src/domains/communications"));
    let frontend_communications_queries =
        read_all_sources(root.join("frontend/src/domains/communications/queries"));
    let frontend_integration_runtime = read_all_sources(root.join("frontend/src/integrations"));
    let frontend_platform_bootstrap =
        read_all_sources(root.join("frontend/src/platform/bootstrap"));
    let frontend_layout_scopes = read(root.join("frontend/src/shared/stores/layoutEditor.ts"));
    let legacy_telegram_key = format!("['{}'", "telegram");
    let legacy_whatsapp_key = format!("['{}'", "whatsapp");
    assert!(
        !frontend_communications_queries.contains(&legacy_telegram_key),
        "user-facing communication caches must not use provider-rooted Telegram query keys"
    );
    assert!(
        !frontend_communications_queries.contains(&legacy_whatsapp_key),
        "user-facing communication caches must not use provider-rooted WhatsApp query keys"
    );
    for forbidden_business_key in [
        "['integrations', 'telegram', 'messages'",
        "['integrations', 'telegram', 'chats'",
        "['integrations', 'whatsapp', 'messages'",
    ] {
        assert!(
            !frontend_integration_runtime.contains(forbidden_business_key),
            "provider business cache key must live under communications, not integrations: {forbidden_business_key}"
        );
    }
    assert!(
        frontend_communications_domain.contains("['communications', 'telegram', 'messages'")
            && frontend_communications_domain.contains("['communications', 'telegram', 'chats'")
            && frontend_communications_domain.contains("['communications', 'whatsapp', 'messages'"),
        "Telegram/WhatsApp business caches must be owned by the Communications domain"
    );
    assert!(
        !frontend_integration_runtime.contains("['communications', 'telegram'")
            && !frontend_integration_runtime.contains("['communications', 'whatsapp'")
            && !frontend_integration_runtime.contains("\"/api/v1/communications/messages")
            && !frontend_integration_runtime.contains("\"/api/v1/communications/conversations")
            && !frontend_integration_runtime.contains("\"/api/v1/communications/search")
            && !frontend_integration_runtime.contains("\"/api/v1/communications/topics"),
        "integration modules must not own Communications business cache keys or business routes"
    );
    assert!(
        frontend_platform_bootstrap
            .contains("domains/communications/queries/realtimeTelegramPatches")
            && frontend_platform_bootstrap
                .contains("domains/communications/queries/realtimeTelegramParticipantPatches")
            && frontend_platform_bootstrap
                .contains("integrations/telegram/queries/realtimeTelegramCommandPatches"),
        "platform realtime bootstrap must compose Communications business patching separately from Telegram integration runtime patching"
    );
    assert!(
        frontend_integration_runtime.contains("['integrations', 'telegram'"),
        "Telegram runtime query keys must be scoped under integrations"
    );
    assert!(
        frontend_integration_runtime.contains("['integrations', 'whatsapp'"),
        "WhatsApp runtime query keys must be scoped under integrations"
    );
    assert!(
        frontend_layout_scopes.contains("viewScope: ['communications', 'telegram']")
            && frontend_layout_scopes.contains("viewScope: ['communications', 'whatsapp']"),
        "communications workspace must keep Telegram and WhatsApp as communication filters/scopes"
    );
}

#[test]
fn app_messaging_handlers_are_thin() {
    let root = repo_root();
    let telegram_handler_root = root.join("backend/src/app/handlers/telegram");
    let telegram_handler_facade = read(root.join("backend/src/app/handlers/telegram.rs"));
    let whatsapp_handler = read(root.join("backend/src/app/handlers/whatsapp.rs"));
    let all_handler_sources = read_all_sources(root.join("backend/src/app/handlers"));
    let app_sources = read_all_sources(root.join("backend/src/app"));
    let provider_runtime_handler_sources =
        read_all_sources(root.join("backend/src/app/provider_runtime_handlers"));

    let telegram_handler_sources = read_all_sources(telegram_handler_root);
    assert!(
        telegram_handler_sources.trim().is_empty(),
        "backend/src/app/handlers/telegram must not contain provider runtime/store implementation files"
    );
    assert!(
        telegram_handler_facade.contains("provider_runtime_handlers::telegram")
            && whatsapp_handler.contains("provider_runtime_handlers::whatsapp"),
        "messaging app handlers must be thin facades over the provider runtime composition root"
    );
    for forbidden in [
        "telegram_store(",
        "whatsapp_store(",
        "crate::integrations::telegram::client::lifecycle",
    ] {
        assert!(
            !all_handler_sources.contains(forbidden),
            "app handlers must not call provider runtime/store helper directly: {forbidden}"
        );
    }
    for forbidden in [
        "telegram_store(",
        "whatsapp_store(",
        "crate::integrations::telegram::client",
        "crate::integrations::whatsapp::client",
        "TelegramStore",
        "WhatsappWebStore",
    ] {
        assert!(
            !app_sources.contains(forbidden),
            "backend/src/app must not own concrete provider client/store code: {forbidden}"
        );
    }
    let telegram_facade_sources = telegram_handler_facade + &whatsapp_handler;
    assert!(
        !telegram_facade_sources.contains("crate::integrations::"),
        "Telegram/WhatsApp handler facades must not import integrations directly"
    );
    for forbidden in [
        "telegram_runtime_store(",
        "whatsapp_runtime_store(",
        "crate::integrations::telegram::client",
        "crate::integrations::telegram::runtime",
        "crate::integrations::telegram::tdjson",
        "crate::integrations::whatsapp::client",
        "TelegramStore",
        "WhatsappWebStore",
    ] {
        assert!(
            !provider_runtime_handler_sources.contains(forbidden),
            "provider runtime handlers must call application services/contracts instead of provider implementations: {forbidden}"
        );
    }
}

#[test]
fn whatsapp_provider_runtime_is_replaceable_trait_boundary() {
    let root = repo_root();
    let runtime_source = read_all_sources(root.join("backend/src/integrations/whatsapp/runtime"));
    let application_contracts =
        read(root.join("backend/src/application/provider_runtime_contracts.rs"));
    let application_services =
        read(root.join("backend/src/application/provider_runtime_services.rs"));

    assert!(
        runtime_source.contains("pub trait WhatsAppProviderRuntime"),
        "WhatsApp runtime boundary must expose the replaceable provider trait"
    );
    for required_method in [
        "fn runtime_status",
        "fn start_runtime",
        "fn stop_runtime",
        "fn runtime_health",
        "fn start_qr_link",
        "fn start_pair_code_link",
        "fn request_send_text",
        "fn request_reply",
        "fn request_forward",
        "fn request_edit",
        "fn request_delete",
        "fn request_react",
        "fn request_unreact",
        "fn request_media_upload",
        "fn request_media_download",
        "fn request_mark_read",
        "fn request_mark_unread",
        "fn request_archive",
        "fn request_unarchive",
        "fn request_mute",
        "fn request_unmute",
        "fn request_pin",
        "fn request_unpin",
        "fn request_join_group",
        "fn request_leave_group",
        "fn request_publish_status",
        "fn request_send_voice_note",
        "fn execute_live_provider_command",
        "fn list_provider_commands",
        "fn manual_retry_provider_command",
        "fn dead_letter_provider_command",
        "fn store_authorized_session_credential",
        "fn ingest_fixture_dialog",
        "fn ingest_fixture_participant",
        "fn ingest_fixture_message_update",
        "fn ingest_fixture_message_delete",
        "fn ingest_fixture_receipt",
        "fn ingest_fixture_reaction",
        "fn ingest_fixture_media",
        "fn ingest_fixture_status",
    ] {
        assert!(
            runtime_source.contains(required_method),
            "WhatsApp provider runtime trait must include {required_method}"
        );
    }
    assert!(
        runtime_source.contains("impl WhatsAppProviderRuntime for WhatsappWebStore"),
        "current WhatsApp Web fixture adapter must implement the provider runtime trait"
    );
    assert!(
        runtime_source.contains("vault: &'a HostVault"),
        "WhatsApp runtime lifecycle must receive host vault context so session restore does not bypass the provider trait"
    );
    assert!(
        runtime_source.contains("WhatsAppProviderExecutableCommand")
            && runtime_source.contains("WhatsAppProviderCommandExecutionOutcome")
            && runtime_source.contains("WhatsAppProviderCommandExecutionError"),
        "WhatsApp live provider command execution must use a replaceable runtime DTO/outcome contract"
    );
    assert!(
        runtime_source.contains("fn start_qr_link<'a>(")
            && runtime_source.contains("fn start_pair_code_link<'a>(")
            && runtime_source.contains("secret_store: &'a SecretReferenceStore")
            && runtime_source.contains("vault: &'a HostVault"),
        "WhatsApp QR/pair-code linking must receive secret-store and host-vault context before native live auth can create restorable sessions"
    );
    assert!(
        runtime_source.contains("ProviderAccountSecretPurpose::WhatsappWebSessionKey")
            && runtime_source.contains("SecretStoreKind::HostVault"),
        "successful WhatsApp authorization must store account-scoped session material in host vault"
    );
    assert!(
        application_contracts.contains("WhatsAppProviderRuntimeRef"),
        "application contracts must expose a trait-object runtime reference"
    );
    assert!(
        application_contracts.contains("whatsapp_provider_runtime_mux("),
        "application runtime factory must compose WhatsApp provider runtimes through a shape-aware mux"
    );
    assert!(
        application_contracts.contains("WhatsappRuntimeSignalIngestService::new")
            && application_contracts.contains("whatsapp_native_md_runtime("),
        "application runtime factory must wire native_md through the shared runtime event sink instead of a direct domain callback"
    );
    assert!(
        !application_contracts.contains("WhatsappWebStore as WhatsappProviderRuntimeStore"),
        "application contracts must not alias the concrete WhatsApp Web store as the runtime"
    );
    assert!(
        !application_contracts.contains("WhatsappWebStore::new"),
        "application contracts must construct WhatsApp runtime through integration runtime factory"
    );
    assert!(
        runtime_source.contains("struct ShapedWhatsAppProviderRuntime"),
        "shape-specific WhatsApp runtime delegates must preserve provider_shape through the runtime boundary"
    );
    assert!(
        runtime_source.contains("WhatsAppProviderRuntimeShape::NativeMultiDevice")
            && runtime_source.contains("WhatsAppProviderRuntimeShape::BusinessCloud"),
        "native_md and business_cloud runtime delegates must exist as explicit provider-shape constructors"
    );
    assert!(
        runtime_source.contains("native_md_manager: Option<native_md::NativeMdRuntimeManager>")
            && runtime_source.contains("with_native_md_manager")
            && runtime_source.contains(".start_runtime(self.inner.as_ref()")
            && runtime_source.contains(".start_qr_link(self.inner.as_ref()")
            && runtime_source.contains(".start_pair_code_link(self.inner.as_ref()")
            && runtime_source.contains("manager.stop_account(&request.account_id)")
            && runtime_source.contains(".decorate_runtime_health(&mut health, account_id)"),
        "shape-specific native_md runtime must keep account lifecycle orchestration behind the provider runtime trait"
    );
    assert!(
        runtime_source.contains("fn all_runtimes(&self) -> [Arc<dyn WhatsAppProviderRuntime>; 3]"),
        "runtime mux must expose provider-family iteration through the trait boundary instead of assuming a single concrete runtime"
    );
    assert!(
        runtime_source.contains("async fn aggregate_sessions(")
            && runtime_source.contains("async fn aggregate_recent_messages("),
        "runtime mux must aggregate accountless WhatsApp reads across provider shapes"
    );
    assert!(
        runtime_source.contains("async fn manual_retry_across_runtimes(")
            && runtime_source.contains("async fn dead_letter_across_runtimes("),
        "runtime mux must route command state transitions across provider runtimes instead of hardcoding one store"
    );
    assert!(
        !runtime_source.contains("let runtime = self.default_runtime();"),
        "runtime mux must not fall back to an implicit default runtime for provider-command retries or accountless reads"
    );
    assert!(
        application_services.contains("runtime: WhatsAppProviderRuntimeRef"),
        "application service must depend on the WhatsApp provider runtime trait object"
    );
    assert!(
        !application_services.contains("store: WhatsappProviderRuntimeStore"),
        "application service must not depend on a concrete WhatsApp runtime store"
    );
}

#[test]
fn whatsapp_provider_libraries_are_confined_to_runtime_boundary() {
    let root = repo_root();
    let backend_src_root = root.join("backend/src");
    let native_runtime_root = root.join("backend/src/integrations/whatsapp/runtime/native_md");
    let native_runtime_file = root.join("backend/src/integrations/whatsapp/runtime/native_md.rs");
    let business_cloud_runtime_root =
        root.join("backend/src/integrations/whatsapp/runtime/business_cloud");
    let backend_cargo = read(root.join("backend/Cargo.toml"));
    let runtime_source = read(root.join("backend/src/integrations/whatsapp/runtime/mod.rs"));
    let runtime_contracts_source =
        read(root.join("backend/src/integrations/whatsapp/runtime/contracts.rs"));
    let application_contracts_source =
        read(root.join("backend/src/application/provider_runtime_contracts.rs"));
    let application_bootstrap_source = read(root.join("backend/src/application/bootstrap.rs"));
    let command_executor_source =
        read(root.join("backend/src/application/whatsapp_command_executor.rs"));
    let runtime_signal_ingest_source =
        read(root.join("backend/src/application/whatsapp_runtime_signal_ingest.rs"));
    let messaging_routes_source = read(root.join("backend/src/app/router/routes/messaging.rs"));
    let native_runtime_source =
        read(root.join("backend/src/integrations/whatsapp/runtime/native_md.rs"));
    let business_cloud_runtime_source =
        read(root.join("backend/src/integrations/whatsapp/runtime/business_cloud.rs"));
    let mut forbidden_hits = Vec::new();
    collect_forbidden_whatsapp_provider_imports(
        &backend_src_root,
        &native_runtime_root,
        &native_runtime_file,
        &business_cloud_runtime_root,
        &mut forbidden_hits,
    );

    assert!(
        forbidden_hits.is_empty(),
        "third-party WhatsApp provider imports must stay inside their runtime adapter boundary: {}",
        forbidden_hits.join(", ")
    );
    assert!(
        backend_cargo.contains("whatsapp-native-md-runtime = [\"dep:wa-rs\", \"dep:wa-rs-core\"]")
            && backend_cargo.contains("async-trait = \"0.1\"")
            && backend_cargo.contains("wa-rs = {")
            && backend_cargo.contains("wa-rs-core = {")
            && backend_cargo.contains("rust-version = \"1.89\"")
            && backend_cargo.contains("default-features = false")
            && backend_cargo.contains("\"tokio-transport\"")
            && backend_cargo.contains("\"ureq-client\"")
            && !backend_cargo.contains("\"sqlite-storage\"")
            && !backend_cargo.contains("wa-rs-sqlite-storage")
            && !backend_cargo.contains("whatsapp-rust ="),
        "native_md must use the selected wa-rs compile boundary without enabling whatsapp-rust or SDK SQLite storage in backend dependencies"
    );
    assert!(
        native_runtime_source.contains("wa_rs::bot::Bot")
            && !native_runtime_source.contains("whatsapp_rust"),
        "native_md runtime is the only backend source allowed to reference the selected wa-rs SDK compile guard"
    );
    assert!(
        native_runtime_source.contains("enum NativeMdDriverReadiness")
            && native_runtime_source.contains("NativeMdDriverReadiness::SmokeGatedUnverified")
            && native_runtime_source.contains("smoke_gated_unverified_public_blocked")
            && native_runtime_source.contains("whatsapp_native_md_public_availability_blocked")
            && native_runtime_source
                .contains("session_secret_purpose: \"whatsapp_web_session_key\""),
        "native_md must expose a smoke-gated driver descriptor without claiming public live availability"
    );
    assert!(
        native_runtime_source.contains("struct NativeMdRuntimeActor")
            && native_runtime_source.contains("struct NativeMdRuntimeActorContract")
            && native_runtime_source.contains("NativeMdRuntimeCommandChannel::DurableOutbox")
            && native_runtime_source.contains("NativeMdRuntimeEventSink::SignalHubRawEvidence")
            && native_runtime_source.contains("NativeMdRuntimeLiveCapabilities::blocked()")
            && native_runtime_source.contains("NativeMdWaRsStoreFamily::SignalStore")
            && native_runtime_source.contains("NativeMdWaRsStoreFamily::AppSyncStore")
            && native_runtime_source.contains("NativeMdWaRsStoreFamily::ProtocolStore")
            && native_runtime_source.contains("NativeMdWaRsStoreFamily::DeviceStore")
            && native_runtime_source.contains("NativeMdWaRsStoreManifest::host_vault_backend")
            && native_runtime_source.contains("struct NativeMdHostVaultBackend")
            && native_runtime_source
                .contains("impl wa_rs::store::SignalStore for NativeMdHostVaultBackend")
            && native_runtime_source
                .contains("impl wa_rs::store::AppSyncStore for NativeMdHostVaultBackend")
            && native_runtime_source
                .contains("impl wa_rs::store::ProtocolStore for NativeMdHostVaultBackend")
            && native_runtime_source
                .contains("impl wa_rs::store::DeviceStore for NativeMdHostVaultBackend")
            && native_runtime_source.contains("store_secret(")
            && native_runtime_source.contains("read_secret(")
            && native_runtime_source.contains("provider_account_session")
            && native_runtime_source.contains("host_vault_encrypted_snapshot")
            && native_runtime_source.contains("struct NativeMdWaRsClientFactory")
            && native_runtime_source.contains("NativeMdWaRsClientFactory::configured_builder")
            && native_runtime_source
                .contains("wa_rs::transport::TokioWebSocketTransportFactory::new()")
            && native_runtime_source.contains("wa_rs::transport::UreqHttpClient::new()")
            && native_runtime_source.contains("native_md_wa_rs_provider_event_id")
            && native_runtime_source.contains("sanitized_dto_only_no_domain_calls")
            && runtime_contracts_source.contains("pub struct WhatsAppSanitizedRuntimeEventDto")
            && runtime_contracts_source.contains("pub trait WhatsAppRuntimeEventSink")
            && runtime_contracts_source.contains("WhatsAppRuntimeEventSinkError")
            && runtime_contracts_source.contains("assert_event_spine_contract")
            && native_runtime_source.contains("native_md_owned_sanitized_runtime_event_dto")
            && native_runtime_source.contains("Arc<dyn WhatsAppRuntimeEventSink>")
            && native_runtime_source.contains("struct NativeMdSanitizedEventCaptureSink")
            && native_runtime_source.contains("owned_sanitized_dto_event_spine_sink")
            && native_runtime_source.contains("struct NativeMdTransientAuthArtifacts")
            && native_runtime_source.contains(".record_event(&event_account_id, &event)")
            && native_runtime_source.contains("memory_only_not_postgres_events_logs")
            && native_runtime_source.contains("whatsapp_native_md_qr_link_artifact_transient")
            && native_runtime_source.contains("whatsapp_native_md_pair_code_artifact_transient")
            && native_runtime_source.contains("native_md_render_transient_qr_svg")
            && runtime_signal_ingest_source.contains("struct WhatsappRuntimeSignalIngestService")
            && runtime_signal_ingest_source.contains("impl WhatsAppRuntimeEventSink")
            && runtime_signal_ingest_source.contains("record_raw_source(&raw)")
            && runtime_signal_ingest_source.contains("dispatch_whatsapp_raw_signal")
            && runtime_signal_ingest_source.contains("redact_secret_like_metadata")
            && native_runtime_source.contains("struct NativeMdLiveDriver")
            && native_runtime_source.contains("NativeMdLiveDriver::build")
            && native_runtime_source.contains("NativeMdLiveDriver::start")
            && native_runtime_source.contains("NativeMdLiveDriver::stop")
            && native_runtime_source.contains("builder.build().await")
            && native_runtime_source.contains(".run().await")
            && native_runtime_source.contains(".disconnect().await")
            && native_runtime_source.contains("tokio::task::JoinHandle<()>")
            && native_runtime_source
                .contains("build_then_run_disconnect_abort_no_public_availability")
            && native_runtime_source.contains("struct NativeMdRuntimeManager")
            && native_runtime_source.contains("NATIVE_MD_LIVE_SMOKE_OPT_IN_CONFIG_KEY")
            && native_runtime_source.contains("native_md_live_smoke_enabled")
            && native_runtime_source.contains("whatsapp_native_md_live_smoke_opt_in_required")
            && native_runtime_source.contains("whatsapp_native_md_public_availability_blocked")
            && native_runtime_source.contains("ensure_link_session_binding")
            && native_runtime_source.contains("link_start_creates_host_vault_binding")
            && native_runtime_source
                .contains("whatsapp_native_md_qr_link_started_event_spine_pending")
            && native_runtime_source
                .contains("whatsapp_native_md_pair_code_link_started_event_spine_pending")
            && native_runtime_source.contains("NativeMdHostVaultBackendSnapshot::default()")
            && native_runtime_source.contains("authorization_state")
            && native_runtime_source.contains("\"linking\"")
            && native_runtime_source.contains("blocked_until_manual_live_smoke")
            && native_runtime_source.contains("NATIVE_MD_VERIFIED_PROVIDER_COMMANDS")
            && native_runtime_source.contains("NATIVE_MD_UNSUPPORTED_PROVIDER_COMMANDS")
            && native_runtime_source.contains("smoke_gated_provider_observed_reconciliation")
            && native_runtime_source.contains("verified_provider_command_subset")
            && native_runtime_source.contains("unsupported_provider_commands")
            && native_runtime_source.contains("verified_sdk_download_path_smoke_gated")
            && native_runtime_source.contains("start_policy")
            && native_runtime_source.contains("explicit_account_config_smoke_opt_in")
            && native_runtime_source.contains("active_account_count")
            && native_runtime_source.contains("struct NativeMdRuntimeLifecycleRegistry")
            && native_runtime_source.contains("NATIVE_MD_RECONNECT_BASE_DELAY_SECONDS")
            && native_runtime_source.contains("NATIVE_MD_RECONNECT_MAX_ATTEMPTS")
            && native_runtime_source.contains("record_provider_event")
            && native_runtime_source.contains("record_reconnect_started")
            && native_runtime_source.contains("record_reconnect_failed")
            && native_runtime_source.contains("reconnect_due")
            && native_runtime_source.contains("connection.degraded")
            && native_runtime_source.contains("connection.recovered")
            && native_runtime_source.contains("connection.reconnect.started")
            && native_runtime_source.contains("native_md_synthetic_runtime_lifecycle_dto")
            && native_runtime_source.contains("tick_driven_reconnect_from_vault_bound_session")
            && native_runtime_source.contains("native_md_execute_provider_command")
            && native_runtime_source.contains(".send_message(")
            && native_runtime_source.contains(".edit_message(")
            && native_runtime_source.contains(".revoke_message(")
            && native_runtime_source.contains(".mark_as_read(")
            && native_runtime_source.contains(".groups()")
            && native_runtime_source.contains(".leave(&chat)")
            && native_runtime_source.contains(".upload(")
            && native_runtime_source.contains("\"send_media\" | \"send_voice_note\"")
            && native_runtime_source.contains("native_md_wa_rs_sanitized_media_download_refs")
            && native_runtime_source.contains("native_md_materialize_media_download_refs")
            && native_runtime_source.contains("native_md_store_media_download_ref")
            && native_runtime_source.contains("provider_media_download_ref")
            && native_runtime_source.contains("host_vault_only_not_postgres_events_logs_frontend")
            && native_runtime_source.contains("SecretReferenceStore")
            && native_runtime_source.contains("native_md_wa_rs_media_ref_metadata")
            && native_runtime_source.contains("host_vault_secret_purpose")
            && native_runtime_source.contains("whatsapp_media_download_ref")
            && native_runtime_source
                .contains("metadata_hashes_only_no_media_key_direct_path_url_or_bytes")
            && native_runtime_source.contains("\"media_key\": \"excluded\"")
            && native_runtime_source.contains("\"direct_path\": \"excluded\"")
            && native_runtime_source.contains("\"static_url\": \"excluded\"")
            && native_runtime_source.contains("native_md_media_upload_message")
            && native_runtime_source.contains("native_md_media_upload_operation_metadata")
            && native_runtime_source.contains("\"media_key\": \"excluded\"")
            && native_runtime_source.contains("media_key_sha256")
            && native_runtime_source.contains("provider_observed_event_reconciliation_required")
            && native_runtime_source.contains("native_md_provider_sdk_command_failed")
            && native_runtime_source.contains("native_md_command_kind_unsupported")
            && runtime_source.contains("claim_due_native_md_commands_for_execution")
            && runtime_source.contains("record_live_provider_command_submitted")
            && runtime_source.contains("locked_at = NULL")
            && runtime_source
                .contains("account.config->>'provider_shape', '') = 'whatsapp_native_md'")
            && command_executor_source.contains("execute_due_live_native_md_commands")
            && command_executor_source.contains("execute_live_provider_command(&executable)")
            && command_executor_source.contains("prepare_live_native_md_media_upload")
            && command_executor_source.contains("prepare_live_native_md_media_download")
            && command_executor_source.contains("persist_live_native_md_media_download")
            && command_executor_source.contains("whatsapp_native_md_media_download_secret_ref")
            && command_executor_source.contains("LocalCommunicationBlobStore::new")
            && command_executor_source.contains("DEFAULT_MAIL_SYNC_BLOB_ROOT")
            && command_executor_source.contains("upload_media_sha256")
            && command_executor_source.contains("submitted_to_provider_awaiting_observed_evidence")
            && command_executor_source.contains("record_live_provider_command_submitted")
            && command_executor_source.contains("provider_observed_event_reconciliation_required")
            && runtime_contracts_source.contains("WhatsAppProviderInMemoryMediaBytes")
            && runtime_contracts_source.contains("#[serde(skip_serializing)]")
            && runtime_contracts_source.contains(".field(\"payload\", &\"redacted\")")
            && application_bootstrap_source.contains("execute_due_live_native_md_commands")
            && runtime_signal_ingest_source.contains("sanitized_runtime_state_override")
            && runtime_signal_ingest_source.contains("allowed_runtime_status")
            && runtime_signal_ingest_source.contains("\"media_key\"")
            && runtime_signal_ingest_source.contains("\"direct_path\"")
            && runtime_signal_ingest_source.contains("\"static_url\"")
            && native_runtime_source.contains("direct_domain_calls")
            && native_runtime_source.contains("forbidden")
            && native_runtime_source.contains("metadata_binding_only")
            && native_runtime_source.contains("sdk_sqlite_policy: \"disabled\"")
            && native_runtime_source.contains("postgres_secret_policy: \"forbidden\"")
            && native_runtime_source
                .contains("fn native_md_runtime_driver_health_check() -> Value")
            && native_runtime_source.contains("\"driver_id\": actor.driver_id()")
            && native_runtime_source.contains("\"command_channel\": actor.command_channel()")
            && native_runtime_source.contains("\"event_sink\": actor.event_sink()")
            && native_runtime_source
                .contains("\"session_secret_purpose\": actor.session_secret_purpose()")
            && native_runtime_source.contains("\"wa_rs_store_manifest\": actor")
            && native_runtime_source.contains("\"restore_scope\": \"account_scoped\"")
            && application_contracts_source.contains("WhatsappRuntimeSignalIngestService::new")
            && application_contracts_source.contains("whatsapp_runtime_event_sink")
            && runtime_source.contains("checks[\"native_md_driver\"]")
            && native_runtime_source.contains("checks[\"native_md_manager\"]")
            && runtime_source.contains("checks[\"runtime\"][\"native_driver\"]"),
        "native_md must define an account-scoped actor contract that routes commands through the durable outbox and events through Signal Hub raw evidence"
    );
    assert!(
        native_runtime_source.contains("wa_rs::bot::Bot::builder().skip_history_sync()")
            && native_runtime_source.contains("wa_rs::types::events::Event")
            && native_runtime_source.contains("classify_wa_rs_event")
            && native_runtime_source.contains("classify_wa_rs_message_event")
            && native_runtime_source.contains("message_has_media_payload")
            && native_runtime_source.contains("NativeMdRawEvidenceEnvelope")
            && native_runtime_source.contains("native_md_wa_rs_raw_evidence_envelope")
            && native_runtime_source.contains("NativeMdSanitizedProviderEventDto")
            && native_runtime_source.contains("native_md_wa_rs_sanitized_event_dto")
            && native_runtime_source.contains("native_md_wa_rs_sanitized_metadata")
            && native_runtime_source.contains("WhatsAppRuntimeBridgeDispatch")
            && native_runtime_source.contains("assert_runtime_bridge_contract")
            && native_runtime_source.contains("dyn wa_rs::store::Backend")
            && native_runtime_source.contains("std::any::type_name::<NativeMdHostVaultBackend>()")
            && native_runtime_source.contains("wa_rs_core::store::Device")
            && native_runtime_source.contains("dyn wa_rs::transport::TransportFactory")
            && native_runtime_source.contains("wa_rs::transport::TokioWebSocketTransportFactory")
            && native_runtime_source.contains("dyn wa_rs::http::HttpClient")
            && native_runtime_source.contains("wa_rs::transport::UreqHttpClient")
            && native_runtime_source.contains("wa_rs::store::Device")
            && native_runtime_source.contains("wa_rs::Client")
            && native_runtime_source.contains("wa_rs::bot::Bot")
            && native_runtime_source.contains("wa_rs::pair_code::PairCodeOptions"),
        "native_md compile probe must bind to the real wa-rs bot, event, storage, transport, HTTP, pair-code and sanitized inbound DTO API surface inside the adapter boundary"
    );
    let verified_native_commands = source_between(
        &native_runtime_source,
        "const NATIVE_MD_VERIFIED_PROVIDER_COMMANDS",
        "const NATIVE_MD_UNSUPPORTED_PROVIDER_COMMANDS",
    );
    let unsupported_native_commands = source_between(
        &native_runtime_source,
        "const NATIVE_MD_UNSUPPORTED_PROVIDER_COMMANDS",
        "const NATIVE_MD_PUBLIC_AVAILABILITY_GATE",
    );
    assert!(
        verified_native_commands.contains("\"download_media\"")
            && !unsupported_native_commands.contains("\"download_media\""),
        "native_md download_media must stay in the smoke-gated verified command subset and must not regress to unsupported"
    );
    for required_wa_rs_event_variant in [
        "Event::PairSuccess",
        "Event::PairError",
        "Event::PairingQrCode",
        "Event::PairingCode",
        "Event::Connected",
        "Event::Disconnected",
        "Event::Message",
        "Event::Receipt",
        "Event::ChatPresence",
        "Event::Presence",
        "Event::JoinedGroup",
        "Event::GroupInfoUpdate",
        "Event::ContactUpdate",
        "Event::HistorySync",
        "Event::OfflineSyncPreview",
        "Event::OfflineSyncCompleted",
        "Event::Notification",
        "Event::BusinessStatusUpdate",
        "Event::ConnectFailure",
        "Event::StreamError",
    ] {
        assert!(
            native_runtime_source.contains(required_wa_rs_event_variant),
            "native_md must classify wa-rs provider event variant {required_wa_rs_event_variant}"
        );
    }
    assert!(
        native_runtime_source.contains("reaction_message.is_some()")
            && native_runtime_source.contains("enc_reaction_message.is_some()")
            && native_runtime_source.contains("protocol_message.r#type")
            && native_runtime_source.contains("Type::Revoke")
            && native_runtime_source.contains("Type::MessageEdit")
            && native_runtime_source.contains("NativeMdProviderEventClassification::unsupported")
            && native_runtime_source.contains("unsupported_evidence"),
        "native_md wa-rs event classifier must preserve reactions, edits, deletes and unknown/provider-only events as Hub evidence instead of dropping them"
    );
    assert!(
        native_runtime_source.contains("provider_shape: \"whatsapp_native_md\"")
            && native_runtime_source.contains("runtime_driver: \"wa-rs\"")
            && native_runtime_source.contains("source_fingerprint:v5:")
            && native_runtime_source
                .contains("NativeMdRawEvidencePayloadPolicy::SanitizedMetadataOnly")
            && native_runtime_source.contains("no_session_token_cookie_or_raw_secret")
            && native_runtime_source.contains("no_message_body_in_runtime_metadata")
            && native_runtime_source.contains("no_media_bytes_in_postgres_events_or_logs")
            && native_runtime_source.contains("\"message_body\": \"excluded\"")
            && native_runtime_source.contains("\"media_bytes\": \"excluded\"")
            && native_runtime_source.contains("\"session_material\": \"excluded\"")
            && native_runtime_source.contains("\"raw_provider_payload\": \"excluded\"")
            && native_runtime_source.contains("\"qr_code\": \"excluded\"")
            && native_runtime_source.contains("\"pair_code\": \"excluded\"")
            && native_runtime_source.contains("\"raw_node\": \"excluded\"")
            && native_runtime_source.contains("\"protobuf_action\": \"excluded\"")
            && native_runtime_source.contains("\"history_sync_payload\": \"excluded\""),
        "native_md raw evidence envelope must be account-scoped, idempotent, sanitized and forbidden from carrying session secrets, message bodies or media bytes"
    );
    for required_raw_record_kind in [
        "whatsapp_web_message",
        "whatsapp_web_message_update",
        "whatsapp_web_message_delete",
        "whatsapp_web_reaction",
        "whatsapp_web_receipt",
        "whatsapp_web_media",
        "whatsapp_web_presence",
        "whatsapp_web_dialog",
        "whatsapp_web_participant",
        "whatsapp_web_call",
        "whatsapp_web_runtime_event",
    ] {
        assert!(
            native_runtime_source.contains(required_raw_record_kind),
            "native_md event classifier must route to raw record kind {required_raw_record_kind}"
        );
    }
    for required_raw_signal_event_kind in [
        "signal.raw.whatsapp.message.observed",
        "signal.raw.whatsapp.message_update.observed",
        "signal.raw.whatsapp.message_delete.observed",
        "signal.raw.whatsapp.reaction.observed",
        "signal.raw.whatsapp.receipt.observed",
        "signal.raw.whatsapp.media.observed",
        "signal.raw.whatsapp.presence.observed",
        "signal.raw.whatsapp.dialog.observed",
        "signal.raw.whatsapp.participant.observed",
        "signal.raw.whatsapp.call_metadata.observed",
        "signal.raw.whatsapp.runtime_event.observed",
    ] {
        assert!(
            native_runtime_source.contains(required_raw_signal_event_kind),
            "native_md raw evidence envelope must route to Signal Hub raw event kind {required_raw_signal_event_kind}"
        );
    }
    for required_runtime_bridge_endpoint in [
        "/api/v1/integrations/whatsapp/runtime-bridge/messages",
        "/api/v1/integrations/whatsapp/runtime-bridge/message-updates",
        "/api/v1/integrations/whatsapp/runtime-bridge/message-deletes",
        "/api/v1/integrations/whatsapp/runtime-bridge/receipts",
        "/api/v1/integrations/whatsapp/runtime-bridge/reactions",
        "/api/v1/integrations/whatsapp/runtime-bridge/media",
        "/api/v1/integrations/whatsapp/runtime-bridge/media-lifecycle",
        "/api/v1/integrations/whatsapp/runtime-bridge/statuses",
        "/api/v1/integrations/whatsapp/runtime-bridge/status-views",
        "/api/v1/integrations/whatsapp/runtime-bridge/status-deletes",
        "/api/v1/integrations/whatsapp/runtime-bridge/presence",
        "/api/v1/integrations/whatsapp/runtime-bridge/calls",
        "/api/v1/integrations/whatsapp/runtime-bridge/dialogs",
        "/api/v1/integrations/whatsapp/runtime-bridge/participants",
        "/api/v1/integrations/whatsapp/runtime-bridge/runtime-events",
        "/api/v1/integrations/whatsapp/runtime-bridge/sync-lifecycle",
    ] {
        assert!(
            native_runtime_source.contains(required_runtime_bridge_endpoint),
            "native_md sanitized DTO dispatch must target existing runtime bridge endpoint {required_runtime_bridge_endpoint}"
        );
        assert!(
            messaging_routes_source.contains(required_runtime_bridge_endpoint),
            "runtime bridge route must exist for native_md dispatch endpoint {required_runtime_bridge_endpoint}"
        );
    }
    for required_runtime_bridge_source in [
        "provider_observed.runtime_bridge_message",
        "provider_observed.runtime_bridge_message_update",
        "provider_observed.runtime_bridge_message_delete",
        "provider_observed.runtime_bridge_receipt",
        "provider_observed.runtime_bridge_reaction",
        "provider_observed.runtime_bridge_media",
        "provider_observed.runtime_bridge_media_lifecycle",
        "provider_observed.runtime_bridge_status",
        "provider_observed.runtime_bridge_status_view",
        "provider_observed.runtime_bridge_status_delete",
        "provider_observed.runtime_bridge_presence",
        "provider_observed.runtime_bridge_call",
        "provider_observed.runtime_bridge_dialog",
        "provider_observed.runtime_bridge_participant",
        "provider_observed.runtime_bridge_runtime_event",
        "provider_observed.runtime_bridge_sync_lifecycle",
    ] {
        assert!(
            native_runtime_source.contains(required_runtime_bridge_source),
            "native_md dispatch must preserve live runtime observed-source marker {required_runtime_bridge_source}"
        );
    }
    for required_event_family in [
        "NativeMdProviderEventFamily::Authentication",
        "NativeMdProviderEventFamily::RuntimeLifecycle",
        "NativeMdProviderEventFamily::SyncLifecycle",
        "NativeMdProviderEventFamily::Message",
        "NativeMdProviderEventFamily::MessageUpdate",
        "NativeMdProviderEventFamily::MessageDelete",
        "NativeMdProviderEventFamily::Receipt",
        "NativeMdProviderEventFamily::Reaction",
        "NativeMdProviderEventFamily::Dialog",
        "NativeMdProviderEventFamily::Participant",
        "NativeMdProviderEventFamily::Presence",
        "NativeMdProviderEventFamily::CallMetadata",
        "NativeMdProviderEventFamily::Status",
        "NativeMdProviderEventFamily::StatusView",
        "NativeMdProviderEventFamily::StatusDelete",
        "NativeMdProviderEventFamily::Media",
        "NativeMdProviderEventFamily::MediaLifecycle",
        "NativeMdProviderEventFamily::CommandReconciliation",
        "NativeMdProviderEventFamily::Unsupported",
    ] {
        assert!(
            native_runtime_source.contains(required_event_family),
            "native_md actor contract must cover provider event family {required_event_family}"
        );
    }
    assert!(
        runtime_source.contains("native_md::native_md_live_runtime_enabled()")
            && native_runtime_source.contains("fn native_md_live_runtime_enabled() -> bool")
            && native_runtime_source.contains("false")
            && runtime_source.contains("native_md::native_md_runtime_feature_blocker()")
            && runtime_source.contains("qr_pair_code_blockers(&runtime_kind, provider_shape)")
            && runtime_source.contains(
                "provider_command_blockers(&runtime_kind, provider_shape, session_restore_available)"
            )
            && runtime_source.contains("business_cloud::business_cloud_live_runtime_enabled()")
            && business_cloud_runtime_source
                .contains("fn business_cloud_live_runtime_enabled() -> bool")
            && business_cloud_runtime_source.contains("false"),
        "compile features must not mark native_md or business_cloud live runtimes available until real drivers exist"
    );
    assert!(
        runtime_source.contains(
            "COALESCE(account.config->>'runtime', '') NOT IN ('', 'fixture', 'live_blocked')"
        ) && runtime_source.contains(
            "COALESCE(command.audit_metadata->>'session_restore_available', 'false') = 'true'"
        ) && runtime_source.contains("COALESCE(account.config->>'lifecycle_state', '') IN ("),
        "live runtime bridge command claim must not execute commands without vault-backed session evidence or for fixture, live_blocked or unlinked WhatsApp accounts"
    );
    assert!(
        application_bootstrap_source.contains("start_whatsapp_runtime_restore_reconciliation")
            && application_bootstrap_source
                .contains("restore_whatsapp_runtime_from_vault_session_if_enabled")
            && application_bootstrap_source
                .contains("should_start_whatsapp_runtime_from_restored_session")
            && application_bootstrap_source.contains("native_md_startup_restore_enabled")
            && application_bootstrap_source
                .contains("runtime.start_runtime(secret_store, vault, &request).await")
            && application_bootstrap_source.contains("WhatsAppRuntimeStartRequest")
            && application_bootstrap_source.contains("startup_restore_start")
            && application_bootstrap_source.contains("startup_restore_start_failed")
            && application_bootstrap_source.contains("whatsapp_startup_restore_failed"),
        "startup restore reconciliation must start eligible native_md runtimes from vault-bound sessions through the replaceable runtime trait"
    );
}

#[test]
fn whatsapp_runtime_has_no_direct_domain_or_engine_write_boundary() {
    let root = repo_root();
    let whatsapp_integration_sources =
        read_all_sources(root.join("backend/src/integrations/whatsapp"));
    let whatsapp_application_sources = read_all_sources(root.join("backend/src/application"));
    let domain_engine_workflow_sources = read_all_sources(root.join("backend/src/domains"))
        + &read_all_sources(root.join("backend/src/engines"))
        + &read_all_sources(root.join("backend/src/workflows"));

    for forbidden in [
        "crate::domains::persons",
        "crate::domains::tasks",
        "crate::domains::documents",
        "crate::domains::memory",
        "crate::engines::search",
        "crate::engines::timeline",
        "crate::engines::consistency",
        "ReviewInbox",
        "TaskCandidate",
        "DecisionCandidate",
        "Obligation",
        "MemoryObservation",
    ] {
        assert!(
            !whatsapp_integration_sources.contains(forbidden),
            "WhatsApp runtime/integration must publish evidence/events, not write domain or engine state directly: {forbidden}"
        );
    }

    for forbidden in [
        "crate::integrations::whatsapp",
        "WhatsAppProviderRuntime",
        "WhatsappWebStore",
        "whatsapp_provider_runtime",
    ] {
        assert!(
            !domain_engine_workflow_sources.contains(forbidden),
            "domains/engines/workflows must not depend on WhatsApp runtime/provider implementation: {forbidden}"
        );
    }

    assert!(
        whatsapp_application_sources.contains("whatsapp_runtime_event_projection")
            && whatsapp_application_sources
                .contains("whatsapp_provider_observation_reconciliation"),
        "WhatsApp application layer must consume event-spine observations through projection/reconciliation workers"
    );
}

#[test]
fn canonical_communication_migration_exists_after_0148() {
    let root = repo_root();
    let migration =
        read(root.join("backend/migrations/0149_create_canonical_communication_tables.sql"));
    let canonical_migration_corpus = migration.clone()
        + &read(root.join("backend/migrations/0157_create_whatsapp_provider_write_commands.sql"));

    for table_name in [
        "communication_accounts",
        "communication_channels",
        "communication_identities",
        "communication_conversations",
        "communication_conversation_participants",
        "communication_message_versions",
        "communication_message_tombstones",
        "communication_message_reactions",
        "communication_message_refs",
        "communication_folders",
        "communication_drafts",
        "communication_outbox",
        "communication_provider_commands",
        "communication_sync_runs",
        "communication_sync_checkpoints",
        "communication_raw_payloads",
    ] {
        assert!(
            migration.contains(&format!("CREATE TABLE IF NOT EXISTS {table_name}")),
            "canonical migration must create {table_name}"
        );
    }

    for legacy_source in [
        "FROM communication_provider_accounts",
        "FROM telegram_message_versions",
        "FROM telegram_message_tombstones",
        "FROM telegram_message_reactions",
        "FROM telegram_message_reply_refs",
        "FROM telegram_message_forward_refs",
        "FROM mail_folders",
        "FROM mail_saved_searches",
        "FROM email_drafts",
        "FROM email_outbox_tracking",
        "FROM telegram_provider_write_commands",
        "FROM whatsapp_provider_write_commands",
        "FROM communication_mail_sync_runs",
        "FROM communication_ingestion_checkpoints",
        "FROM communication_raw_records",
    ] {
        assert!(
            canonical_migration_corpus.contains(legacy_source),
            "canonical migration must migrate legacy source {legacy_source}"
        );
    }
}

#[test]
fn whatsapp_docs_bundle_exists_for_acceptance_artifacts() {
    let root = repo_root();
    let whatsapp_readme = read(root.join("docs/integrations/whatsapp/README.md"));
    let fixture_matrix = read(root.join("docs/integrations/whatsapp/fixture-test-matrix.md"));
    let smoke_checklist = read(root.join("docs/integrations/whatsapp/live-smoke-checklist.md"));
    let api_reference = read(root.join("docs/integrations/whatsapp/api.md"));

    assert!(
        whatsapp_readme.contains("fixture-test-matrix.md")
            && whatsapp_readme.contains("live-smoke-checklist.md")
            && whatsapp_readme.contains("api.md"),
        "WhatsApp README must reference the current API, fixture-test matrix and live smoke checklist artifacts"
    );
    assert!(
        fixture_matrix.contains("Fixture tests cover every source record and command class.")
            && fixture_matrix.contains("Retried background executor coverage"),
        "WhatsApp fixture-test matrix must document acceptance coverage for source records and command classes"
    );
    assert!(
        smoke_checklist.contains("live runtime")
            || smoke_checklist.contains("Live runtime")
            || smoke_checklist.contains("runtime"),
        "WhatsApp live smoke checklist must exist as the manual live validation artifact"
    );
    assert!(
        api_reference.contains("/api/v1/communications/messages")
            && api_reference.contains("/api/v1/integrations/whatsapp/provider-commands"),
        "WhatsApp API reference must describe provider-neutral reads plus integration control routes"
    );
}

#[test]
fn communication_runtime_core_uses_canonical_storage_tables() {
    let root = repo_root();
    for (path, forbidden) in [
        (
            "backend/src/domains/communications/ai_state.rs",
            vec!["mail_ai_states"],
        ),
        (
            "backend/src/domains/communications/drafts.rs",
            vec!["email_drafts"],
        ),
        (
            "backend/src/domains/communications/folders.rs",
            vec!["mail_folders", "mail_folder_messages"],
        ),
        (
            "backend/src/domains/communications/outbox.rs",
            vec!["email_outbox_tracking", "mail_read_receipts"],
        ),
        (
            "backend/src/domains/communications/templates.rs",
            vec!["email_templates"],
        ),
        (
            "backend/src/domains/communications/rules/store.rs",
            vec!["email_rules"],
        ),
        (
            "backend/src/domains/communications/finance.rs",
            vec!["email_invoices"],
        ),
        (
            "backend/src/domains/communications/legal.rs",
            vec!["email_legal_documents"],
        ),
        (
            "backend/src/domains/communications/personas.rs",
            vec!["email_personas"],
        ),
        (
            "backend/src/domains/communications/signatures/store.rs",
            vec!["email_certificates"],
        ),
        (
            "backend/src/domains/communications/read_receipts.rs",
            vec!["email_outbox_tracking", "mail_read_receipts"],
        ),
        (
            "backend/src/domains/communications/saved_searches.rs",
            vec!["mail_saved_searches"],
        ),
    ] {
        let source = read(root.join(path));
        for needle in forbidden {
            assert!(
                !source.contains(needle),
                "{path} must not read or write legacy storage table {needle}"
            );
        }
    }

    let read_receipts_migration =
        read(root.join("backend/migrations/0151_create_communication_read_receipts.sql"));
    assert!(
        read_receipts_migration.contains("CREATE TABLE IF NOT EXISTS communication_read_receipts"),
        "read receipt canonical migration must create communication_read_receipts"
    );
    assert!(
        read_receipts_migration.contains("FROM mail_read_receipts"),
        "read receipt canonical migration must backfill from mail_read_receipts"
    );

    let aux_migration =
        read(root.join("backend/migrations/0152_create_canonical_communication_aux_tables.sql"));
    for table_name in [
        "communication_rules",
        "communication_templates",
        "communication_personas",
        "communication_invoices",
        "communication_legal_documents",
        "communication_certificates",
    ] {
        assert!(
            aux_migration.contains(&format!("CREATE TABLE IF NOT EXISTS {table_name}")),
            "aux canonical migration must create {table_name}"
        );
    }
    for legacy_source in [
        "FROM email_rules",
        "FROM email_templates",
        "FROM email_personas",
        "FROM email_invoices",
        "FROM email_legal_documents",
        "FROM email_certificates",
    ] {
        assert!(
            aux_migration.contains(legacy_source),
            "aux canonical migration must backfill from {legacy_source}"
        );
    }
}

#[test]
fn communications_domain_uses_canonical_dto_names() {
    let root = repo_root();
    let backend_domain = read_all_sources(root.join("backend/src/domains/communications"));
    let frontend_domain = read_all_sources(root.join("frontend/src/domains/communications"));

    for legacy_name in [
        "EmailDraft",
        "NewEmailDraft",
        "EmailTemplate",
        "NewEmailTemplate",
        "EmailPersona",
        "NewEmailPersona",
        "EmailOutboxItem",
        "NewEmailOutboxItem",
        "EmailOutboxStatus",
        "EmailThread",
        "MailThreadSummary",
        "EmailSearchResponse",
        "SendEmailRequest",
        "SendEmailResponse",
        "EmailFinanceStore",
        "EmailExport",
    ] {
        assert!(
            !backend_domain.contains(legacy_name),
            "backend communications domain must not expose legacy DTO name {legacy_name}"
        );
        assert!(
            !frontend_domain.contains(legacy_name),
            "frontend communications domain must not expose legacy DTO name {legacy_name}"
        );
    }

    for canonical_name in [
        "CommunicationDraft",
        "CommunicationTemplate",
        "CommunicationPersona",
        "CommunicationOutboxItem",
        "CommunicationOutboxStatus",
        "CommunicationThread",
        "CommunicationThreadSummary",
        "CommunicationSearchResponse",
        "SendCommunicationRequest",
        "SendCommunicationResponse",
    ] {
        assert!(
            backend_domain.contains(canonical_name) || frontend_domain.contains(canonical_name),
            "communications domain must expose canonical DTO name {canonical_name}"
        );
    }
}

fn read(path: PathBuf) -> String {
    fs::read_to_string(&path).unwrap_or_else(|error| {
        panic!("failed to read {}: {error}", path.display());
    })
}

fn read_all_sources(root: PathBuf) -> String {
    let mut output = String::new();
    collect_sources(&root, &mut output);
    output
}

fn source_between<'a>(source: &'a str, start_marker: &str, end_marker: &str) -> &'a str {
    let start = source.find(start_marker).unwrap_or_else(|| {
        panic!("source marker `{start_marker}` is missing");
    });
    let end = source[start..]
        .find(end_marker)
        .map(|offset| start + offset)
        .unwrap_or_else(|| {
            panic!("source marker `{end_marker}` is missing after `{start_marker}`");
        });
    &source[start..end]
}

fn collect_sources(path: &Path, output: &mut String) {
    if path.is_file() {
        if matches!(
            path.extension().and_then(|value| value.to_str()),
            Some("rs" | "ts" | "vue" | "js" | "mjs")
        ) {
            output.push_str(&read(path.to_path_buf()));
            output.push('\n');
        }
        return;
    }

    let Ok(entries) = fs::read_dir(path) else {
        return;
    };
    for entry in entries.flatten() {
        collect_sources(&entry.path(), output);
    }
}

fn collect_forbidden_whatsapp_provider_imports(
    path: &Path,
    native_runtime_root: &Path,
    native_runtime_file: &Path,
    business_cloud_runtime_root: &Path,
    hits: &mut Vec<String>,
) {
    if path.is_file() {
        if path.extension().and_then(|value| value.to_str()) != Some("rs") {
            return;
        }
        let content = read(path.to_path_buf());
        let native_runtime_allowed =
            path.starts_with(native_runtime_root) || path == native_runtime_file;
        for native_forbidden in ["whatsapp_rust", "wa_rs", "whatsapp-rust", "wa-rs"] {
            if content.contains(native_forbidden) && !native_runtime_allowed {
                hits.push(format!("{}:{native_forbidden}", path.display()));
            }
        }
        for business_forbidden in ["whatsapp_business_rs", "wacloudapi"] {
            if content.contains(business_forbidden)
                && !path.starts_with(business_cloud_runtime_root)
            {
                hits.push(format!("{}:{business_forbidden}", path.display()));
            }
        }
        return;
    }

    let Ok(entries) = fs::read_dir(path) else {
        return;
    };
    for entry in entries.flatten() {
        collect_forbidden_whatsapp_provider_imports(
            &entry.path(),
            native_runtime_root,
            native_runtime_file,
            business_cloud_runtime_root,
            hits,
        );
    }
}
