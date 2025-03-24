use std::fmt::Debug;

use async_trait::async_trait;
use chrono::{DateTime, TimeZone, Utc};
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::forex::{
    entity::{HistoricalRates, Order, Rates, RatesData, RatesList, RatesResponse},
    interface::{ForexHistoricalRates, ForexRates, ForexStorage},
    Currency, ForexResult,
};

use super::Money;

fn latest_rate() -> Rates {
    let latest_update = Utc.with_ymd_and_hms(2025, 3, 4, 2, 0, 0).unwrap();
    let base = Currency::USD;
    let rates = RatesData {
        usd: dec!(1),
        idr: dec!(16461),
        eur: dec!(0.953416),
        gbp: dec!(0.787563),
        jpy: dec!(148.9353),
        chf: dec!(0.89583),
        sgd: dec!(1.344868),
        cny: dec!(7.286),
        sar: dec!(3.750387),
        xau: dec!(0.0003462),
        xag: dec!(0.03165459),
        xpt: dec!(0.00104119),

        // Additional currencies and assets
        cad: dec!(1.273),
        rub: dec!(93.5),
        krw: dec!(1320.5),
        hkd: dec!(7.84),
        myr: dec!(4.69),
        thb: dec!(35.2),
        aed: dec!(3.6725),
        kwd: dec!(0.306),
        inr: dec!(83.1),
        aud: dec!(1.52),
        nzd: dec!(1.67),
        btc: dec!(0.0000158),
        eth: dec!(0.00049),
        sol: dec!(0.0117),
        xrp: dec!(1.92),
        ada: dec!(3.76),
    };

    Rates {
        latest_update,
        base,
        rates,
    }
}

fn historical_rate() -> HistoricalRates {
    let date = Utc.with_ymd_and_hms(2022, 12, 25, 0, 0, 0).unwrap();
    let base = Currency::USD;
    let rates = RatesData {
        usd: dec!(1),
        idr: dec!(15588.665563),
        eur: dec!(0.941531),
        gbp: dec!(0.829531),
        jpy: dec!(132.80956357),
        chf: dec!(0.93335),
        sgd: dec!(1.350445),
        cny: dec!(6.98946),
        sar: dec!(3.7603),
        xau: dec!(0.00055331),
        xag: dec!(0.04211858),
        xpt: dec!(0.0009742),

        // Additional currencies and assets
        cad: dec!(1.273),
        rub: dec!(93.5),
        krw: dec!(1320.5),
        hkd: dec!(7.84),
        myr: dec!(4.69),
        thb: dec!(35.2),
        aed: dec!(3.6725),
        kwd: dec!(0.306),
        inr: dec!(83.1),
        aud: dec!(1.52),
        nzd: dec!(1.67),
        btc: dec!(0.0000158),
        eth: dec!(0.00049),
        sol: dec!(0.0117),
        xrp: dec!(1.92),
        ada: dec!(3.76),
    };

    HistoricalRates { date, base, rates }
}

fn latest_rate_list(page: u32, size: u32, order: Order) -> RatesList<RatesResponse<Rates>> {
    let mut rates_list: Vec<RatesResponse<Rates>> = vec![
        RatesResponse {
            id: Uuid::parse_str("10324ad3-1caa-4acc-9296-a7b34a6ad010").unwrap(),
            source: "openexchangerates.org".to_string(),
            poll_date: "2025-03-04T01:35:07.792048Z"
                .parse::<DateTime<Utc>>()
                .unwrap(),
            data: Rates {
                latest_update: "2025-03-04T01:00:00Z".parse::<DateTime<Utc>>().unwrap(),
                base: Currency::USD,
                rates: RatesData {
                    usd: dec!(1.0),
                    idr: dec!(16461.0),
                    eur: dec!(0.953435),
                    gbp: dec!(0.787419),
                    jpy: dec!(149.157125),
                    chf: dec!(0.896309),
                    sgd: dec!(1.345818),
                    cny: dec!(7.2851),
                    sar: dec!(3.750418),
                    xau: dec!(0.00034576),
                    xag: dec!(0.03156671),
                    xpt: dec!(0.00103929),

                    // Additional currencies and assets
                    cad: dec!(1.273),
                    rub: dec!(93.5),
                    krw: dec!(1320.5),
                    hkd: dec!(7.84),
                    myr: dec!(4.69),
                    thb: dec!(35.2),
                    aed: dec!(3.6725),
                    kwd: dec!(0.306),
                    inr: dec!(83.1),
                    aud: dec!(1.52),
                    nzd: dec!(1.67),
                    btc: dec!(0.0000158),
                    eth: dec!(0.00049),
                    sol: dec!(0.0117),
                    xrp: dec!(1.92),
                    ada: dec!(3.76),
                },
            },
            error: None,
        },
        RatesResponse {
            id: Uuid::parse_str("51d5a6fd-a83c-4fec-980b-e5faae6fc1fa").unwrap(),
            source: "openexchangerates.org".to_string(),
            poll_date: "2025-03-04T02:32:02.165177Z"
                .parse::<DateTime<Utc>>()
                .unwrap(),
            data: Rates {
                latest_update: "2025-03-04T02:00:00Z".parse::<DateTime<Utc>>().unwrap(),
                base: Currency::USD,
                rates: RatesData {
                    usd: dec!(1.0),
                    idr: dec!(16461.0),
                    eur: dec!(0.953416),
                    gbp: dec!(0.787563),
                    jpy: dec!(148.9353),
                    chf: dec!(0.89583),
                    sgd: dec!(1.344868),
                    cny: dec!(7.286),
                    sar: dec!(3.750387),
                    xau: dec!(0.0003462),
                    xag: dec!(0.03165459),
                    xpt: dec!(0.00104119),

                    // Newly added fields
                    cad: dec!(1.25),
                    rub: dec!(92.5),
                    krw: dec!(1315.75),
                    hkd: dec!(7.83),
                    myr: dec!(4.68),
                    thb: dec!(36.15),
                    aed: dec!(3.67),
                    kwd: dec!(0.31),
                    inr: dec!(82.85),
                    aud: dec!(1.52),
                    nzd: dec!(1.62),
                    btc: dec!(0.000023),
                    eth: dec!(0.00031),
                    sol: dec!(0.0045),
                    xrp: dec!(1.1),
                    ada: dec!(3.2),
                },
            },
            error: None,
        },
        RatesResponse {
            id: Uuid::parse_str("c385aea1-8e79-4028-b44c-bf26450fc457").unwrap(),
            source: "openexchangerates.org".to_string(),
            poll_date: "2025-03-03T11:27:48.272544Z"
                .parse::<DateTime<Utc>>()
                .unwrap(),
            data: Rates {
                latest_update: "2025-03-03T11:00:00Z".parse::<DateTime<Utc>>().unwrap(),
                base: Currency::USD,
                rates: RatesData {
                    usd: dec!(1.0),
                    idr: dec!(16439.272482),
                    eur: dec!(0.957671),
                    gbp: dec!(0.790732),
                    jpy: dec!(150.8345),
                    chf: dec!(0.90168),
                    sgd: dec!(1.348395),
                    cny: dec!(7.2907),
                    sar: dec!(3.750414),
                    xau: dec!(0.00034822),
                    xag: dec!(0.03176984),
                    xpt: dec!(0.00104974),

                    // Newly added fields
                    cad: dec!(1.25),
                    rub: dec!(92.5),
                    krw: dec!(1315.75),
                    hkd: dec!(7.83),
                    myr: dec!(4.68),
                    thb: dec!(36.15),
                    aed: dec!(3.67),
                    kwd: dec!(0.31),
                    inr: dec!(82.85),
                    aud: dec!(1.52),
                    nzd: dec!(1.62),
                    btc: dec!(0.000023),
                    eth: dec!(0.00031),
                    sol: dec!(0.0045),
                    xrp: dec!(1.1),
                    ada: dec!(3.2),
                },
            },
            error: None,
        },
        RatesResponse {
            id: Uuid::parse_str("1f5624b0-58ad-40d5-9122-6896d80eec53").unwrap(),
            source: "openexchangerates.org".to_string(),
            poll_date: "2025-03-03T10:44:39.072957Z"
                .parse::<DateTime<Utc>>()
                .unwrap(),
            data: Rates {
                latest_update: "2025-03-03T10:00:00Z".parse::<DateTime<Utc>>().unwrap(),
                base: Currency::USD,
                rates: RatesData {
                    usd: dec!(1.0),
                    idr: dec!(16473.71557),
                    eur: dec!(0.959016),
                    gbp: dec!(0.791831),
                    jpy: dec!(150.3485),
                    chf: dec!(0.900817),
                    sgd: dec!(1.34789),
                    cny: dec!(7.289),
                    sar: dec!(3.750438),
                    xau: dec!(0.00034874),
                    xag: dec!(0.03183193),
                    xpt: dec!(0.00105358),

                    // Newly added fields
                    cad: dec!(1.25),
                    rub: dec!(92.5),
                    krw: dec!(1315.75),
                    hkd: dec!(7.83),
                    myr: dec!(4.68),
                    thb: dec!(36.15),
                    aed: dec!(3.67),
                    kwd: dec!(0.31),
                    inr: dec!(82.85),
                    aud: dec!(1.52),
                    nzd: dec!(1.62),
                    btc: dec!(0.000023),
                    eth: dec!(0.00031),
                    sol: dec!(0.0045),
                    xrp: dec!(1.1),
                    ada: dec!(3.2),
                },
            },
            error: None,
        },
        RatesResponse {
            id: Uuid::parse_str("d95447d8-3935-49d6-855d-d2585365adf0").unwrap(),
            source: "openexchangerates.org".to_string(),
            poll_date: "2025-02-28T23:32:29.225381Z"
                .parse::<DateTime<Utc>>()
                .unwrap(),
            data: Rates {
                latest_update: "2025-02-28T23:00:04Z".parse::<DateTime<Utc>>().unwrap(),
                base: Currency::USD,
                rates: RatesData {
                    usd: dec!(1.0),
                    idr: dec!(16531.45),
                    eur: dec!(0.96355),
                    gbp: dec!(0.795355),
                    jpy: dec!(150.61499887),
                    chf: dec!(0.9033),
                    sgd: dec!(1.3513),
                    cny: dec!(7.2838),
                    sar: dec!(3.750582),
                    xau: dec!(0.0003499),
                    xag: dec!(0.03210067),
                    xpt: dec!(0.00106384),

                    // Additional fields
                    cad: dec!(1.25),
                    rub: dec!(92.5),
                    krw: dec!(1315.75),
                    hkd: dec!(7.83),
                    myr: dec!(4.68),
                    thb: dec!(36.15),
                    aed: dec!(3.67),
                    kwd: dec!(0.31),
                    inr: dec!(82.85),
                    aud: dec!(1.52),
                    nzd: dec!(1.62),
                    btc: dec!(0.000023),
                    eth: dec!(0.00031),
                    sol: dec!(0.0045),
                    xrp: dec!(1.1),
                    ada: dec!(3.2),
                },
            },
            error: None,
        },
        RatesResponse {
            id: Uuid::parse_str("421d55b4-c3e5-49fb-a816-b89f78a0f275").unwrap(),
            source: "openexchangerates.org".to_string(),
            poll_date: "2025-02-24T05:30:29.120432Z"
                .parse::<DateTime<Utc>>()
                .unwrap(),
            data: Rates {
                latest_update: "2025-02-24T05:00:00Z".parse::<DateTime<Utc>>().unwrap(),
                base: Currency::USD,
                rates: RatesData {
                    usd: dec!(1.0),
                    idr: dec!(16297.031896),
                    eur: dec!(0.950973),
                    gbp: dec!(0.788796),
                    jpy: dec!(149.213),
                    chf: dec!(0.895933),
                    sgd: dec!(1.33233),
                    cny: dec!(7.2348),
                    sar: dec!(3.7501),
                    xau: dec!(0.00034007),
                    xag: dec!(0.03058717),
                    xpt: dec!(0.00101452),

                    // Additional fields
                    cad: dec!(1.25),
                    rub: dec!(92.5),
                    krw: dec!(1315.75),
                    hkd: dec!(7.83),
                    myr: dec!(4.68),
                    thb: dec!(36.15),
                    aed: dec!(3.67),
                    kwd: dec!(0.31),
                    inr: dec!(82.85),
                    aud: dec!(1.52),
                    nzd: dec!(1.62),
                    btc: dec!(0.000023),
                    eth: dec!(0.00031),
                    sol: dec!(0.0045),
                    xrp: dec!(1.1),
                    ada: dec!(3.2),
                },
            },
            error: None,
        },
        RatesResponse {
            id: Uuid::parse_str("df80eeda-2552-416e-b1ab-a40e9558beab").unwrap(),
            source: "openexchangerates.org".to_string(),
            poll_date: "2025-02-23T10:22:14.306079Z"
                .parse::<DateTime<Utc>>()
                .unwrap(),
            data: Rates {
                latest_update: "2025-02-23T10:00:04Z".parse::<DateTime<Utc>>().unwrap(),
                base: Currency::USD,
                rates: RatesData {
                    usd: dec!(1.0),
                    idr: dec!(16302.1),
                    eur: dec!(0.956114),
                    gbp: dec!(0.791734),
                    jpy: dec!(149.235),
                    chf: dec!(0.897985),
                    sgd: dec!(1.3353),
                    cny: dec!(7.251),
                    sar: dec!(3.750172),
                    xau: dec!(0.0003406),
                    xag: dec!(0.03077023),
                    xpt: dec!(0.00102184),

                    // Additional fields
                    cad: dec!(1.25),
                    rub: dec!(92.5),
                    krw: dec!(1315.75),
                    hkd: dec!(7.83),
                    myr: dec!(4.68),
                    thb: dec!(36.15),
                    aed: dec!(3.67),
                    kwd: dec!(0.31),
                    inr: dec!(82.85),
                    aud: dec!(1.52),
                    nzd: dec!(1.62),
                    btc: dec!(0.000023),
                    eth: dec!(0.00031),
                    sol: dec!(0.0045),
                    xrp: dec!(1.1),
                    ada: dec!(3.2),
                },
            },
            error: None,
        },
        RatesResponse {
            id: Uuid::parse_str("bcc3681b-1452-41f7-af18-ccee5ffcaadb").unwrap(),
            source: "openexchangerates.org".to_string(),
            poll_date: "2025-02-23T06:55:11.890362Z"
                .parse::<DateTime<Utc>>()
                .unwrap(),
            data: Rates {
                latest_update: "2025-02-23T06:00:23Z".parse::<DateTime<Utc>>().unwrap(),
                base: Currency::USD,
                rates: RatesData {
                    usd: dec!(1.0),
                    idr: dec!(16302.1),
                    eur: dec!(0.956114),
                    gbp: dec!(0.791734),
                    jpy: dec!(149.145),
                    chf: dec!(0.897985),
                    sgd: dec!(1.3353),
                    cny: dec!(7.251),
                    sar: dec!(3.74803),
                    xau: dec!(0.0003406),
                    xag: dec!(0.03077023),
                    xpt: dec!(0.00102184),

                    // Additional fields
                    cad: dec!(1.25),
                    rub: dec!(92.5),
                    krw: dec!(1315.75),
                    hkd: dec!(7.83),
                    myr: dec!(4.68),
                    thb: dec!(36.15),
                    aed: dec!(3.67),
                    kwd: dec!(0.31),
                    inr: dec!(82.85),
                    aud: dec!(1.52),
                    nzd: dec!(1.62),
                    btc: dec!(0.000023),
                    eth: dec!(0.00031),
                    sol: dec!(0.0045),
                    xrp: dec!(1.1),
                    ada: dec!(3.2),
                },
            },
            error: None,
        },
    ];

    match order {
        Order::DESC => rates_list.sort_by(|a, b| b.data.latest_update.cmp(&a.data.latest_update)),
        Order::ASC => rates_list.sort_by(|a, b| a.data.latest_update.cmp(&b.data.latest_update)),
    }

    let start = (page.saturating_sub(1) * size) as usize;
    let end = (start + size as usize).min(rates_list.len());

    let has_prev = start > 0;
    let paginated_rates_list = rates_list[start..end].to_vec();
    let has_next = end < rates_list.len(); // If there's more data beyond this page

    RatesList {
        has_prev,
        rates_list: paginated_rates_list,
        has_next,
    }
}

fn historical_rate_list(
    page: u32,
    size: u32,
    order: Order,
) -> RatesList<RatesResponse<HistoricalRates>> {
    let mut historical_rates_list: Vec<RatesResponse<HistoricalRates>> = vec![
        RatesResponse {
            id: Uuid::parse_str("d06e8e1c-6d64-4bd4-98d6-2758bcbf2d5f").unwrap(),
            source: "openexchangerates.org".to_string(),
            poll_date: "2025-03-04T06:31:27.111458Z"
                .parse::<DateTime<Utc>>()
                .unwrap(),
            data: HistoricalRates {
                date: "2022-12-25T23:59:39Z".parse::<DateTime<Utc>>().unwrap(),
                base: Currency::USD,
                rates: RatesData {
                    usd: dec!(1.0),
                    idr: dec!(15588.665563),
                    eur: dec!(0.941531),
                    gbp: dec!(0.829531),
                    jpy: dec!(132.80956357),
                    chf: dec!(0.93335),
                    sgd: dec!(1.350445),
                    cny: dec!(6.98946),
                    sar: dec!(3.7603),
                    xau: dec!(0.00055331),
                    xag: dec!(0.04211858),
                    xpt: dec!(0.0009742),

                    // Additional fields
                    cad: dec!(1.25),
                    rub: dec!(92.5),
                    krw: dec!(1315.75),
                    hkd: dec!(7.83),
                    myr: dec!(4.68),
                    thb: dec!(36.15),
                    aed: dec!(3.67),
                    kwd: dec!(0.31),
                    inr: dec!(82.85),
                    aud: dec!(1.52),
                    nzd: dec!(1.62),
                    btc: dec!(0.000023),
                    eth: dec!(0.00031),
                    sol: dec!(0.0045),
                    xrp: dec!(1.1),
                    ada: dec!(3.2),
                },
            },
            error: None,
        },
        RatesResponse {
            id: Uuid::parse_str("7185a19d-55bf-40d6-993d-2d3ee54d0ca4").unwrap(),
            source: "openexchangerates.org".to_string(),
            poll_date: "2025-03-04T06:31:00.874617Z"
                .parse::<DateTime<Utc>>()
                .unwrap(),
            data: HistoricalRates {
                date: "2021-12-20T23:59:59Z".parse::<DateTime<Utc>>().unwrap(),
                base: Currency::USD,
                rates: RatesData {
                    usd: dec!(1.0),
                    idr: dec!(14388.75),
                    eur: dec!(0.886746),
                    gbp: dec!(0.75709),
                    jpy: dec!(113.66591667),
                    chf: dec!(0.92178),
                    sgd: dec!(1.36721),
                    cny: dec!(6.3757),
                    sar: dec!(3.754026),
                    xau: dec!(0.00055823),
                    xag: dec!(0.04492115),
                    xpt: dec!(0.00106659),

                    // Additional fields
                    cad: dec!(1.25),
                    rub: dec!(92.5),
                    krw: dec!(1315.75),
                    hkd: dec!(7.83),
                    myr: dec!(4.68),
                    thb: dec!(36.15),
                    aed: dec!(3.67),
                    kwd: dec!(0.31),
                    inr: dec!(82.85),
                    aud: dec!(1.52),
                    nzd: dec!(1.62),
                    btc: dec!(0.000023),
                    eth: dec!(0.00031),
                    sol: dec!(0.0045),
                    xrp: dec!(1.1),
                    ada: dec!(3.2),
                },
            },
            error: None,
        },
        RatesResponse {
            id: Uuid::parse_str("a31994fe-25bd-41ad-9d05-0684c849d87e").unwrap(),
            source: "openexchangerates.org".to_string(),
            poll_date: "2025-03-04T06:30:08.520417Z"
                .parse::<DateTime<Utc>>()
                .unwrap(),
            data: HistoricalRates {
                date: "2021-07-07T23:59:59Z".parse::<DateTime<Utc>>().unwrap(),
                base: Currency::USD,
                rates: RatesData {
                    usd: dec!(1.0),
                    idr: dec!(14512.7),
                    eur: dec!(0.847952),
                    gbp: dec!(0.724652),
                    jpy: dec!(110.63599465),
                    chf: dec!(0.925721),
                    sgd: dec!(1.349139),
                    cny: dec!(6.473),
                    sar: dec!(3.750498),
                    xau: dec!(0.00055449),
                    xag: dec!(0.03825484),
                    xpt: dec!(0.00091912),

                    // Additional fields
                    cad: dec!(1.25),
                    rub: dec!(92.5),
                    krw: dec!(1315.75),
                    hkd: dec!(7.83),
                    myr: dec!(4.68),
                    thb: dec!(36.15),
                    aed: dec!(3.67),
                    kwd: dec!(0.31),
                    inr: dec!(82.85),
                    aud: dec!(1.52),
                    nzd: dec!(1.62),
                    btc: dec!(0.000023),
                    eth: dec!(0.00031),
                    sol: dec!(0.0045),
                    xrp: dec!(1.1),
                    ada: dec!(3.2),
                },
            },
            error: None,
        },
        RatesResponse {
            id: Uuid::parse_str("198fab12-d078-40bf-b403-057019155971").unwrap(),
            source: "openexchangerates.org".to_string(),
            poll_date: "2025-03-04T01:35:06.452147Z"
                .parse::<DateTime<Utc>>()
                .unwrap(),
            data: HistoricalRates {
                date: "2020-01-01T23:59:58Z".parse::<DateTime<Utc>>().unwrap(),
                base: Currency::USD,
                rates: RatesData {
                    usd: dec!(1.0),
                    idr: dec!(13893.633074),
                    eur: dec!(0.891348),
                    gbp: dec!(0.754603),
                    jpy: dec!(108.72525),
                    chf: dec!(0.967795),
                    sgd: dec!(1.345237),
                    cny: dec!(6.9632),
                    sar: dec!(3.75137),
                    xau: dec!(0.00065859),
                    xag: dec!(0.05588309),
                    xpt: dec!(0.00103628),

                    // Additional fields
                    cad: dec!(1.25),
                    rub: dec!(92.5),
                    krw: dec!(1315.75),
                    hkd: dec!(7.83),
                    myr: dec!(4.68),
                    thb: dec!(36.15),
                    aed: dec!(3.67),
                    kwd: dec!(0.31),
                    inr: dec!(82.85),
                    aud: dec!(1.52),
                    nzd: dec!(1.62),
                    btc: dec!(0.000023),
                    eth: dec!(0.00031),
                    sol: dec!(0.0045),
                    xrp: dec!(1.1),
                    ada: dec!(3.2),
                },
            },
            error: None,
        },
    ];

    match order {
        Order::DESC => historical_rates_list.sort_by(|a, b| b.data.date.cmp(&a.data.date)),
        Order::ASC => historical_rates_list.sort_by(|a, b| a.data.date.cmp(&b.data.date)),
    }

    let start = (page.saturating_sub(1) * size) as usize;
    let end = (start + size as usize).min(historical_rates_list.len());

    let has_prev = start > 0;
    let paginated_historical_rates_list = historical_rates_list[start..end].to_vec();
    let has_next = end < historical_rates_list.len(); // If there's more data beyond this page

    RatesList {
        has_prev,
        rates_list: paginated_historical_rates_list,
        has_next,
    }
}

pub(crate) struct ForexApiSuccessMock;

#[async_trait]
impl ForexRates for ForexApiSuccessMock {
    async fn rates(&self, base: Currency) -> ForexResult<RatesResponse<Rates>> {
        Ok(RatesResponse::new(
            "success_latest_mock".to_string(),
            latest_rate(),
        ))
    }
}

#[async_trait]
impl ForexHistoricalRates for ForexApiSuccessMock {
    async fn historical_rates(
        &self,
        date: DateTime<Utc>,
        base: Currency,
    ) -> ForexResult<RatesResponse<HistoricalRates>> {
        Ok(RatesResponse::new(
            "success_historical_mock".to_string(),
            historical_rate(),
        ))
    }
}

pub(crate) struct ForexStorageSuccessMock;

#[async_trait]
impl ForexStorage for ForexStorageSuccessMock {
    async fn insert_latest<T>(
        &self,
        _date: DateTime<Utc>,
        _rates: &RatesResponse<T>,
    ) -> ForexResult<()>
    where
        T: Debug + Serialize + for<'de> Deserialize<'de> + Send + Sync,
    {
        Ok(())
    }

    async fn get_latest(&self) -> ForexResult<RatesResponse<Rates>> {
        Ok(RatesResponse::new(
            "storage_get_latest_success".to_string(),
            latest_rate(),
        ))
    }

    async fn insert_historical<T>(
        &self,
        _date: DateTime<Utc>,
        _rates: &RatesResponse<T>,
    ) -> ForexResult<()>
    where
        T: Debug + Serialize + for<'de> Deserialize<'de> + Send + Sync,
    {
        Ok(())
    }

    async fn insert_historical_batch(
        &self,
        rates: Vec<RatesResponse<HistoricalRates>>,
    ) -> ForexResult<()> {
        Ok(())
    }

    async fn update_historical_rates_data(
        &self,
        date: DateTime<Utc>,
        new_data: Vec<Money>,
    ) -> ForexResult<RatesResponse<HistoricalRates>> {
        Ok(RatesResponse::new(
            "storage_get_historical_success".to_string(),
            historical_rate(),
        ))
    }

    async fn get_historical(
        &self,
        _date: DateTime<Utc>,
    ) -> ForexResult<RatesResponse<HistoricalRates>> {
        Ok(RatesResponse::new(
            "storage_get_historical_success".to_string(),
            historical_rate(),
        ))
    }

    async fn get_latest_list(
        &self,
        page: u32,
        size: u32,
        order: Order,
    ) -> ForexResult<RatesList<RatesResponse<Rates>>> {
        Ok(latest_rate_list(page, size, order))
    }

    async fn get_historical_list(
        &self,
        page: u32,
        size: u32,
        order: Order,
    ) -> ForexResult<RatesList<RatesResponse<HistoricalRates>>> {
        Ok(historical_rate_list(page, size, order))
    }
}
