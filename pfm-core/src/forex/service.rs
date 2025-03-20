use chrono::{DateTime, Utc};

use super::{
    currency::Currency,
    entity::{ConversionResponse, HistoricalRates, Rates, RatesResponse},
    interface::{ForexError, ForexHistoricalRates, ForexRates, ForexResult, ForexStorage},
    money::Money,
};

pub async fn convert<FS>(storage: &FS, from: Money, to: Currency) -> ForexResult<ConversionResponse>
where
    FS: ForexStorage,
{
    let latest_rates = storage.get_latest().await?;
    if let Some(err) = latest_rates.error {
        return Err(ForexError::internal_error(&err));
    }

    let ret = {
        let res = Money::convert(&latest_rates.data.rates, from, to)?;
        let date = latest_rates.data.latest_update;

        ConversionResponse {
            last_update: date,
            money: res,
        }
    };

    Ok(ret)
}

pub async fn batch_convert<FS>(
    storage: &FS,
    from: Vec<Money>,
    to: Currency,
) -> ForexResult<Vec<ConversionResponse>>
where
    FS: ForexStorage,
{
    let mut results: Vec<ConversionResponse> = vec![];

    for x in from {
        let ret = convert(storage, x, to).await?;

        results.push(ret);
    }

    Ok(results)
}

/// Get rates from 3rd API.
/// Invoked from Cron service.
pub async fn poll_rates<FX, FS>(
    forex: &FX,
    storage: &FS,
    base: Currency,
) -> ForexResult<RatesResponse<Rates>>
where
    FX: ForexRates,
    FS: ForexStorage,
{
    let ret = match forex.rates(base).await {
        Ok(val) => val,
        Err(error) => RatesResponse::<Rates>::err(Utc::now(), error),
    };

    storage.insert_latest(ret.data.latest_update, &ret).await?;

    Ok(ret)
}

/// Get historical rates from 3rd API.
/// Invoked from Cron service.
pub async fn poll_historical_rates<FX, FS>(
    forex: &FX,
    storage: &FS,
    date: DateTime<Utc>,
    base: Currency,
) -> ForexResult<RatesResponse<HistoricalRates>>
where
    FX: ForexHistoricalRates,
    FS: ForexStorage,
{
    let ret = match forex.historical_rates(date, base).await {
        Ok(val) => {
            storage.insert_historical(val.data.date, &val).await?;
            val
        }
        Err(error) => {
            let err = RatesResponse::<HistoricalRates>::err(date, error);
            storage.insert_historical(date, &err).await?;
            err
        }
    };

    Ok(ret)
}
