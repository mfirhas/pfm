use std::sync::LazyLock;

use pfm_core::{
    forex_impl::forex_storage::{self, ForexStorageImpl},
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
pub(crate) struct AppContext<FS> {
    pub forex_storage: FS,
}

static CONTEXT: LazyLock<AppContext<ForexStorageImpl>> = LazyLock::new(|| {
    let forex_storage = forex_storage::ForexStorageImpl::new(global::storage_fs());
    let ctx = AppContext { forex_storage };

    ctx
});

/// get dependencies of pfm-http
pub(crate) fn context() -> AppContext<ForexStorageImpl> {
    CONTEXT.clone()
}
