use std::sync::LazyLock;

use pfm_core::{
    forex_impl::currencybeacon::Api as CurrencyBeaconApi,
    forex_impl::{
        self,
        forex_storage::{self, ForexStorageImpl},
    },
    global,
};
use pfm_utils::config_util;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct AppConfig {
    #[serde(alias = "HTTP_PORT")]
    pub http_port: u16,

    /// if enabled, accessing APIs must provide valid api key
    #[serde(alias = "HTTP_ENABLE_API_KEY")]
    pub enable_api_key: bool,

    /// provided from env var, NOT file
    #[serde(alias = "HTTP_ADMIN_PASSWORD")]
    pub admin_password: String,
}

static CONFIG: LazyLock<AppConfig> = LazyLock::new(|| {
    let cfg =
        config_util::get_config::<AppConfig>("HTTP_").expect("pfm-http failed reading config");
    cfg
});

/// get configs of pfm-http
pub(crate) fn config() -> &'static AppConfig {
    &CONFIG
}

#[derive(Clone)]
pub(crate) struct AppContext<FS, FH> {
    pub forex_storage: FS,
    pub forex_historical: FH,
}

static CONTEXT: LazyLock<AppContext<ForexStorageImpl, CurrencyBeaconApi>> = LazyLock::new(|| {
    let forex_storage = forex_storage::ForexStorageImpl::new(global::storage_fs());
    let forex_historical = forex_impl::currencybeacon::Api::new(
        &global::config().forex_currencybeacon_api_key,
        global::http_client(),
    );
    let ctx = AppContext {
        forex_storage,
        forex_historical,
    };

    ctx
});

/// get dependencies of pfm-http
pub(crate) fn context() -> AppContext<ForexStorageImpl, CurrencyBeaconApi> {
    CONTEXT.clone()
}
