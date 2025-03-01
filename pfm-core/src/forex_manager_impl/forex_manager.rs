use crate::{
    forex::{ConversionResponse, Currency, HttpResponse, Money},
    forex_manager::{ForexManager, ForexManagerError, ForexManagerResult},
};

use anyhow::anyhow;
use async_trait::async_trait;

const ERROR_PREFIX: &str = "[FOREX_MANAGER_IMPL]";

const PFM_CORE_HTTP_BATCH_CONVERT_ENDPOINT: &str = "/forex/batch-convert";

pub struct Api {
    client: reqwest::Client,
    pfm_core_base_url: reqwest::Url,
}

impl Api {
    pub fn new(client: reqwest::Client, pfm_core_base_url: &str) -> ForexManagerResult<Self> {
        let url = reqwest::Url::parse(pfm_core_base_url).map_err(|err| {
            ForexManagerError::Error(anyhow!(
                "{} failed parsing pfm core http server base url: {}",
                ERROR_PREFIX,
                err
            ))
        })?;

        Ok(Self {
            client,
            pfm_core_base_url: url,
        })
    }

    pub async fn batch_convert(
        &self,
        from: String,
        to: Currency,
    ) -> ForexManagerResult<Vec<ConversionResponse>> {
        let convert_endpoint = self
            .pfm_core_base_url
            .join(PFM_CORE_HTTP_BATCH_CONVERT_ENDPOINT)
            .map_err(|err| {
                ForexManagerError::Error(anyhow!(
                    "{} failed parsing forex convert endpoint: {}",
                    ERROR_PREFIX,
                    err
                ))
            })?;

        let convert_from = from;
        let convert_to = to.code();
        let ret = self
            .client
            .get(convert_endpoint)
            .query(&[("from", convert_from.as_str()), ("to", convert_to)])
            .send()
            .await
            .map_err(|err| {
                ForexManagerError::Error(anyhow!(
                    "{} failed invoking forex convert endpoint: {}",
                    ERROR_PREFIX,
                    err
                ))
            })?
            .text()
            .await
            .map_err(|err| {
                ForexManagerError::Error(anyhow!(
                    "{} failed converting to string: {}",
                    ERROR_PREFIX,
                    err
                ))
            })?;

        let resp: HttpResponse<Vec<ConversionResponse>> =
            serde_json::from_str(&ret).map_err(|err| {
                ForexManagerError::Error(anyhow!(
                    "{} failed parsing convert response {}: {}",
                    ERROR_PREFIX,
                    &ret,
                    err
                ))
            })?;

        let ret = if let Some(v) = resp.data {
            v
        } else {
            return Err(ForexManagerError::Error(anyhow!(
                "{} convert api error",
                ERROR_PREFIX
            )));
        };

        Ok(ret)
    }
}

#[async_trait]
impl ForexManager for Api {
    async fn batch_convert(
        &self,
        from: Vec<Money>,
        to: Currency,
    ) -> ForexManagerResult<Vec<ConversionResponse>> {
        let input = from
            .iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
            .join(";");

        let ret = self.batch_convert(input, to).await?;

        Ok(ret)
    }
}
