// forex_manager.rs manages forex in CLIENT side

use std::fmt::Display;

use anyhow::anyhow;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::forex::{entity::ConversionResponse, Currency, Money};

const ERROR_PREFIX: &str = "[FOREX_MANAGER]";

/// Represents a cash asset purchase with associated details.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Cash {
    /// Unique identifier for this record.
    #[serde(rename = "id")]
    pub id: Uuid,

    /// Timestamp when this record was created.
    #[serde(rename = "created_at")]
    pub created_at: DateTime<Utc>,

    /// Timestamp when this record was last updated.
    #[serde(rename = "updated_at")]
    pub updated_at: DateTime<Utc>,

    /// The currency and amount purchased.
    #[serde(rename = "money")]
    pub money: Money,

    /// Description about the asset, tell anything.
    #[serde(rename = "desc")]
    pub desc: Option<String>,

    /// The date and time the purchase was made.
    #[serde(rename = "purchase_date")]
    pub purchase_date: DateTime<Utc>,

    /// The price paid when the purchase was made. In Money: <CODE> <AMOUNT>
    #[serde(rename = "purchase_price")]
    pub purchase_price: Money,

    /// Spot price on the purchase date (in purchase_price currency), fetched from APIs.
    #[serde(rename = "spot_price")]
    pub spot_price: Money,

    #[serde(rename = "purchase_spread")]
    pub purchase_spread: Money,

    #[serde(rename = "purchase_spread_percentage")]
    pub purchase_spread_percentage: Money,

    /// Tax collected when the purchase was made.
    #[serde(rename = "purchase_tax")]
    pub purchase_tax: Money,

    /// Percentage of tax collected when the purchase was made.
    #[serde(rename = "purchase_tax_percentage")]
    pub purchase_tax_percentage: Money,

    /// Additional fees when the purchase was made.
    #[serde(rename = "purchase_fee")]
    pub purchase_fee: Money,

    /// Description about the purchase, could be details of taxes/fees.
    #[serde(rename = "purchase_desc")]
    pub purchase_desc: Option<String>,

    /// Total amount spent purchasing the asset: purchase_price + purchase_tax + purchase_fee
    #[serde(rename = "total_purchase")]
    pub total_purchase: Money,

    /// list of unrealized profit & loss made on sale
    #[serde(rename = "upnl")]
    pub upnl: Option<Vec<Upnl>>,
}

/// Unrealized Profit and Loss of each Cash entry
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Upnl {
    /// Unique identifier for this record.
    #[serde(rename = "id")]
    pub id: Uuid,

    /// Timestamp when this record was created.
    #[serde(rename = "created_at")]
    pub created_at: DateTime<Utc>,

    /// Timestamp when this record was last updated.
    #[serde(rename = "updated_at")]
    pub updated_at: DateTime<Utc>,

    /// The date of UPnL calculation.
    #[serde(rename = "sale_date")]
    pub sale_date: DateTime<Utc>,

    /// The price of the asset if sold.
    #[serde(rename = "sale_price")]
    pub sale_price: Decimal,

    /// Spot price when UPnL is calculated (sale date).
    #[serde(rename = "spot_price")]
    pub spot_price: Decimal,

    /// Amount of tax if sold.
    #[serde(rename = "sale_tax")]
    pub sale_tax: Decimal,

    /// Percentage of tax if sold.
    #[serde(rename = "sale_tax_percentage")]
    pub sale_tax_percentage: Decimal,

    /// Additional fees if sold.
    #[serde(rename = "sale_fee")]
    pub sale_fee: Decimal,

    /// Description of sales, e.g., details of taxes/fees.
    #[serde(rename = "sale_desc")]
    pub sale_desc: String,

    /// Total price of sale.
    #[serde(rename = "total_sale")]
    pub total_sale: Decimal,

    /// The profit or loss made if sold.
    #[serde(rename = "margin")]
    pub margin: Decimal,

    /// Percentage of margin made if sold.
    #[serde(rename = "margin_percentage")]
    pub margin_percentage: Decimal,

    /// Duration spent holding this asset (in days). hold_period = sale_date - purchase_date
    #[serde(rename = "hold_period")]
    pub hold_period: u64,
}

#[async_trait]
pub trait ForexManagerStorage {
    /// insert new record into forex assets
    async fn insert(&self, cash: Cash) -> ForexManagerResult<()>;

    /// get an entry from records
    async fn get(&self, id: Uuid) -> ForexManagerResult<Cash>;

    /// get paginated list of entries
    async fn get_list(
        &self,
        page: u32,
        size: u32,
        order: Order,
    ) -> ForexManagerResult<CashListResponse>;

    /// edit existing forex records
    async fn update(&self, entry: Cash) -> ForexManagerResult<()>;

    /// remove an entry from existing records
    async fn delete(&self, id: Uuid) -> ForexManagerResult<()>;
}

#[async_trait]
pub trait ForexManager {
    async fn batch_convert(
        &self,
        from: Vec<Money>,
        to: Currency,
    ) -> ForexManagerResult<Vec<ConversionResponse>>;
}

/////////////////////////////////////// APIs ///////////////////////////////////////
#[derive(Debug, Serialize, Deserialize)]
pub struct ForexPurchaseParams {
    /// The currency and the amount purchased
    #[serde(rename = "money")]
    pub money: Money,

    /// Description about the asset, tell anything.
    #[serde(rename = "desc", default)]
    pub desc: Option<String>,

    /// The date and time the purchase was made.
    #[serde(rename = "purchase_date")]
    pub purchase_date: DateTime<Utc>,

    /// The price paid when the purchase was made.
    #[serde(rename = "purchase_price")]
    pub purchase_price: Money,

    /// Tax collected when the purchase was made.
    #[serde(rename = "purchase_tax")]
    pub purchase_tax: Money,

    /// Additional fees when the purchase was made.
    #[serde(rename = "purchase_fee")]
    pub purchase_fee: Money,

    /// Description about the purchase, could be details of taxes/fees.
    #[serde(rename = "purchase_desc", default)]
    pub purchase_desc: Option<String>,
}

impl TryFrom<ForexPurchaseParams> for Cash {
    type Error = ForexManagerError;

    fn try_from(value: ForexPurchaseParams) -> Result<Self, Self::Error> {
        todo!()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum Order {
    ASC,
    DESC,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CashListResponse {
    pub has_prev: bool,
    pub cash_list: Vec<Cash>,
    pub has_next: bool,
}

pub type ForexManagerResult<T> = Result<T, ForexManagerError>;

#[derive(Debug)]
pub enum ForexManagerError {
    Error(anyhow::Error),
    StorageError(anyhow::Error),
}

impl Display for ForexManagerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ret = match self {
            Self::Error(err) => err.to_string(),
            Self::StorageError(err) => err.to_string(),
        };
        write!(f, "{}", ret)
    }
}

/// add: add new entry to forex portfolio
pub async fn add<FS>(storage: &FS, params: ForexPurchaseParams) -> ForexManagerResult<()>
where
    FS: ForexManagerStorage,
{
    Ok(storage.insert(params.try_into()?).await?)
}

/// entry: get an entry from records
pub async fn entry<FS>(storage: &FS, id: Uuid) -> ForexManagerResult<Cash>
where
    FS: ForexManagerStorage,
{
    Ok(storage.get(id).await?)
}

/// entry_list: get list of entries
pub async fn entries<FS>(
    storage: &FS,
    page: u32,
    size: u32,
    order: Order,
) -> ForexManagerResult<CashListResponse>
where
    FS: ForexManagerStorage,
{
    Ok(storage.get_list(page, size, order).await?)
}

/// subtract: subtract n amount of money from existing records
/// this will subtract from entry with the same currency from request param
pub async fn subtract<FS>(storage: &FS, amount: Money) -> ForexManagerResult<()>
where
    FS: ForexManagerStorage,
{
    // first find the closes amount and same currency, if same amount or lower, substract it. Find the oldest.
    todo!()
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CashTotal {
    pub money: Money,
    pub rates_latest_update: DateTime<Utc>,
}

/// calculate total amount of cash stored as forex from multiple currencies into currency target
pub async fn total<FS, FX>(
    storage: &FS,
    fx: &FX,
    currency_target: Currency,
) -> ForexManagerResult<CashTotal>
where
    FS: ForexManagerStorage,
    FX: ForexManager,
{
    let mut total = dec!(0);
    let mut latest_updates: Vec<DateTime<Utc>> = vec![];
    let mut page = 1;
    let size = 100;
    let order = Order::ASC;
    loop {
        let ret = storage.get_list(page, size, order).await?;
        if ret.cash_list.is_empty() {
            break;
        }

        let sum = convert_and_sum(fx, currency_target, &ret.cash_list).await?;

        total += sum.0;
        latest_updates.push(sum.1);

        if ret.has_next {
            page += 1;
        } else {
            break;
        }
    }

    if latest_updates.is_empty() {
        return Err(ForexManagerError::Error(anyhow!(
            "{} returned empty",
            ERROR_PREFIX
        )));
    }

    latest_updates.sort_by(|a, b| b.cmp(&a));

    let resp = CashTotal {
        money: Money::new_money(currency_target, total),
        rates_latest_update: latest_updates[0],
    };

    Ok(resp)
}

async fn convert_and_sum(
    fx: &impl ForexManager,
    target_currency: Currency,
    list: &[Cash],
) -> ForexManagerResult<(Decimal, DateTime<Utc>)> {
    let input = list.iter().map(|item| item.money).collect::<Vec<_>>();

    let mut ret = fx.batch_convert(input, target_currency).await?;
    if ret.is_empty() {
        return Err(ForexManagerError::Error(anyhow!(
            "{} returned empty from api call",
            ERROR_PREFIX
        )));
    }
    ret.sort_by(|x, y| y.date.cmp(&x.date));

    let sum = ret.iter().map(|v| v.result.amount()).sum::<Decimal>();

    Ok((sum, ret[0].date))
}
/////////////////////////////////////// APIs(END) ///////////////////////////////////////
