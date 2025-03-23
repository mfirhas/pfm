use chrono::{DateTime, Utc};

use crate::global;

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

pub async fn update_historical_rates_data<FX, FS>(
    forex: &FX,
    storage: &FS,
    date: DateTime<Utc>,
    currencies_to_update: Vec<Currency>,
) -> ForexResult<RatesResponse<HistoricalRates>>
where
    FX: ForexHistoricalRates,
    FS: ForexStorage,
{
    let ret = forex.historical_rates(date, global::BASE_CURRENCY).await?;
    let mut new_rates: Vec<Money> = vec![];
    for c in currencies_to_update {
        match c {
            // fiat

            // north america
            Currency::USD => {
                new_rates.push(Money::USD(ret.data.rates.usd));
            }
            Currency::CAD => {
                new_rates.push(Money::CAD(ret.data.rates.cad));
            }

            // europe
            Currency::EUR => {
                new_rates.push(Money::EUR(ret.data.rates.eur));
            }
            Currency::GBP => {
                new_rates.push(Money::GBP(ret.data.rates.gbp));
            }
            Currency::CHF => {
                new_rates.push(Money::CHF(ret.data.rates.chf));
            }
            Currency::RUB => {
                new_rates.push(Money::RUB(ret.data.rates.rub));
            }

            // east asia
            Currency::CNY => {
                new_rates.push(Money::CNY(ret.data.rates.cny));
            }
            Currency::JPY => {
                new_rates.push(Money::JPY(ret.data.rates.jpy));
            }
            Currency::KRW => {
                new_rates.push(Money::KRW(ret.data.rates.krw));
            }
            Currency::HKD => {
                new_rates.push(Money::HKD(ret.data.rates.hkd));
            }

            // south-east asia
            Currency::IDR => {
                new_rates.push(Money::IDR(ret.data.rates.idr));
            }
            Currency::MYR => {
                new_rates.push(Money::MYR(ret.data.rates.myr));
            }
            Currency::SGD => {
                new_rates.push(Money::SGD(ret.data.rates.sgd));
            }
            Currency::THB => {
                new_rates.push(Money::THB(ret.data.rates.thb));
            }

            // middle-east
            Currency::SAR => {
                new_rates.push(Money::SAR(ret.data.rates.sar));
            }
            Currency::AED => {
                new_rates.push(Money::AED(ret.data.rates.aed));
            }
            Currency::KWD => {
                new_rates.push(Money::KWD(ret.data.rates.kwd));
            }

            // south asia
            Currency::INR => {
                new_rates.push(Money::INR(ret.data.rates.inr));
            }

            // apac
            Currency::AUD => {
                new_rates.push(Money::AUD(ret.data.rates.aud));
            }
            Currency::NZD => {
                new_rates.push(Money::NZD(ret.data.rates.nzd));
            }

            //// precious metals
            Currency::XAU => {
                new_rates.push(Money::XAU(ret.data.rates.xau));
            }
            Currency::XAG => {
                new_rates.push(Money::XAG(ret.data.rates.xag));
            }
            Currency::XPT => {
                new_rates.push(Money::XPT(ret.data.rates.xpt));
            }

            //// crypto
            Currency::BTC => {
                new_rates.push(Money::BTC(ret.data.rates.btc));
            }
            Currency::ETH => {
                new_rates.push(Money::ETH(ret.data.rates.eth));
            }
            Currency::SOL => {
                new_rates.push(Money::SOL(ret.data.rates.sol));
            }
            Currency::XRP => {
                new_rates.push(Money::XRP(ret.data.rates.xrp));
            }
            Currency::ADA => {
                new_rates.push(Money::ADA(ret.data.rates.ada));
            }
        }
    }

    let updated_historical_data = storage
        .update_historical_rates_data(date, new_rates)
        .await?;

    Ok(updated_historical_data)
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
