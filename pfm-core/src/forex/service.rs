use anyhow::Context;
use chrono::{DateTime, Datelike, Utc};
use rust_decimal_macros::dec;
use strum::IntoEnumIterator;
use tracing::instrument;

use crate::{error::AsInternalError, forex::entity::RatesData, global::constants};

use super::{
    currency::Currency,
    entity::{ConversionResponse, Rates, RatesResponse},
    interface::{ForexError, ForexHistoricalRates, ForexRates, ForexResult, ForexStorage},
    money::Money,
};

#[instrument(skip(storage), ret)]
pub async fn get_rates(
    storage: &impl ForexStorage,
    base: Currency,
    date: Option<DateTime<Utc>>,
) -> ForexResult<RatesResponse<Rates>> {
    match (base, date) {
        (constants::BASE_CURRENCY, None) => get_rates_usd_latest(storage).await,
        (constants::BASE_CURRENCY, Some(date)) => get_rates_usd_historical(storage, date).await,
        (base, None) => get_rates_base_latest(storage, base).await,
        (base, Some(date)) => get_rates_base_historical(storage, base, date).await,
    }
}

#[instrument(skip(storage), ret)]
async fn get_rates_usd_latest(storage: &impl ForexStorage) -> ForexResult<RatesResponse<Rates>> {
    let latest_ret = storage
        .get_latest()
        .await
        .context("get latest usd based rates")
        .as_internal_err()?;

    if let Some(err) = latest_ret.error {
        return Err(ForexError::internal_error(err.as_str()));
    }

    Ok(latest_ret)
}

#[instrument(skip(storage), ret)]
async fn get_rates_usd_historical(
    storage: &impl ForexStorage,
    date: DateTime<Utc>,
) -> ForexResult<RatesResponse<Rates>> {
    let now = Utc::now();
    if date.year() == now.year() && date.month() == now.month() && date.day() == now.day() {
        return get_rates_usd_latest(storage).await;
    }

    let historical_rates = storage
        .get_historical(date)
        .await
        .context("get historical usd based rates")
        .as_internal_err()?;

    if let Some(err) = historical_rates.error {
        return Err(ForexError::internal_error(err.as_str()));
    }

    Ok(historical_rates)
}

#[instrument(skip(storage), ret)]
async fn get_rates_base_latest(
    storage: &impl ForexStorage,
    base: Currency,
) -> ForexResult<RatesResponse<Rates>> {
    let usd_based_latest_rates = get_rates_usd_latest(storage).await?;
    let date = usd_based_latest_rates.data.date;
    let mut rates_result: Vec<Money> = vec![];
    for target_curr in Currency::iter() {
        if target_curr != base {
            let ret = Money::convert(
                &usd_based_latest_rates.data.rates,
                Money::new_money(base, dec!(1)),
                target_curr,
            )
            .context("get rates base latest conversion")
            .as_internal_err()?;

            rates_result.push(ret);
        } else {
            rates_result.push(Money::new_money(base, dec!(1)));
        }
    }
    let rates_data: RatesData = rates_result.into();
    let rates = Rates {
        date,
        base,
        rates: rates_data,
    };
    let rates_response = RatesResponse {
        id: usd_based_latest_rates.id,
        source: usd_based_latest_rates.source,
        poll_date: usd_based_latest_rates.poll_date,
        data: rates,
        error: usd_based_latest_rates.error,
    };

    Ok(rates_response)
}

#[instrument(skip(storage), ret)]
async fn get_rates_base_historical(
    storage: &impl ForexStorage,
    base: Currency,
    date: DateTime<Utc>,
) -> ForexResult<RatesResponse<Rates>> {
    let now = Utc::now();
    if date.year() == now.year() && date.month() == now.month() && date.day() == now.day() {
        return get_rates_base_latest(storage, base).await;
    }

    let usd_based_historical_rates = get_rates_usd_historical(storage, date).await?;
    let date = usd_based_historical_rates.data.date;
    let mut rates_result: Vec<Money> = vec![];
    for target_curr in Currency::iter() {
        if target_curr != base {
            let ret = Money::convert(
                &usd_based_historical_rates.data.rates,
                Money::new_money(base, dec!(1)),
                target_curr,
            )
            .context("get rates base latest conversion")
            .as_internal_err()?;

            rates_result.push(ret);
        } else {
            rates_result.push(Money::new_money(base, dec!(1)));
        }
    }
    let rates_data: RatesData = rates_result.into();
    let rates = Rates {
        date,
        base,
        rates: rates_data,
    };
    let rates_response = RatesResponse {
        id: usd_based_historical_rates.id,
        source: usd_based_historical_rates.source,
        poll_date: usd_based_historical_rates.poll_date,
        data: rates,
        error: usd_based_historical_rates.error,
    };

    Ok(rates_response)
}

#[instrument(skip(storage), ret)]
pub async fn convert<FS>(storage: &FS, from: Money, to: Currency) -> ForexResult<ConversionResponse>
where
    FS: ForexStorage,
{
    let latest_rates = storage.get_latest().await?;
    if let Some(_) = latest_rates.error {
        return Err(ForexError::internal_error(
            "latest rates for this time not available at the moment, please try again later",
        ));
    }

    let ret = {
        let res = Money::convert(&latest_rates.data.rates, from, to)?;
        if res.amount() == dec!(0) {
            return Err(ForexError::internal_error(
                "service convert rate not available at the moment",
            ));
        }
        let date = latest_rates.data.date;
        let code = res.format(false);
        let symbol = res.format(true);

        ConversionResponse {
            date,
            from,
            to: res,
            code,
            symbol,
        }
    };

    Ok(ret)
}

#[instrument(skip(storage), ret)]
pub async fn convert_historical(
    storage: &impl ForexStorage,
    from: Money,
    to: Currency,
    date: DateTime<Utc>,
) -> ForexResult<ConversionResponse> {
    let historical_rates = storage.get_historical(date).await?;
    if let Some(_) = historical_rates.error {
        return Err(ForexError::internal_error(
            "historical rates for this date not available, please contact the web master",
        ));
    }
    let converted_money = Money::convert(&historical_rates.data.rates, from, to)?;
    if converted_money.amount() == dec!(0) {
        return Err(ForexError::internal_error(
            "service convert historical rate not available for this date, try again or another date, or contact web master",
        ));
    }
    let code = converted_money.format(false);
    let symbol = converted_money.format(true);

    Ok(ConversionResponse {
        date: historical_rates.data.date,
        from,
        to: converted_money,
        code,
        symbol,
    })
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
        if ret.to.amount() == dec!(0) {
            return Err(ForexError::internal_error(
                format!(
                    "service batch_convert rate for {} is not available at the moment",
                    to.code()
                )
                .as_str(),
            ));
        }

        results.push(ret);
    }

    Ok(results)
}

pub async fn update_historical_rates_data<FX, FS>(
    forex: &FX,
    storage: &FS,
    date: DateTime<Utc>,
    currencies_to_update: Vec<Currency>,
) -> ForexResult<RatesResponse<Rates>>
where
    FX: ForexHistoricalRates,
    FS: ForexStorage,
{
    let historical_data = storage.get_historical(date).await?;
    let base = historical_data.data.base;
    let ret = forex.historical_rates(date, base).await?;
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

    storage.insert_latest(ret.data.date, &ret).await?;

    Ok(ret)
}

/// Get historical rates from 3rd API.
/// Invoked from Cron service.
pub async fn poll_historical_rates<FX, FS>(
    forex: &FX,
    storage: &FS,
    date: DateTime<Utc>,
    base: Currency,
) -> ForexResult<RatesResponse<Rates>>
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
            let err = RatesResponse::<Rates>::err(date, error);
            storage.insert_historical(date, &err).await?;
            err
        }
    };

    Ok(ret)
}
