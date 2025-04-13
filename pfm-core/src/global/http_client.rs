use anyhow::Context;
use anyhow::Result;
use reqwest::Client;
use std::sync::LazyLock;
use std::time::Duration;

pub fn http_client() -> Client {
    HTTP_CLIENT.clone()
}

static HTTP_CLIENT: LazyLock<Client> =
    LazyLock::new(|| init_http_client().expect("global static init http client"));

fn init_http_client() -> Result<reqwest::Client, anyhow::Error> {
    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .pool_idle_timeout(Duration::from_secs(300))
        .pool_max_idle_per_host(32)
        .build()
        .context("global: failed initializing http client");

    client
}
