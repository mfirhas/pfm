use strum::IntoEnumIterator;

use super::{entity::RatesData, Currency, Money};

#[test]
fn test_rates_data_fields() {
    let rates_data = RatesData {
        ..Default::default()
    };

    let ret = match rates_data {
        RatesData {
            usd,
            cad,
            eur,
            gbp,
            chf,
            rub,
            cny,
            jpy,
            krw,
            hkd,
            idr,
            myr,
            sgd,
            thb,
            sar,
            aed,
            kwd,
            inr,
            aud,
            nzd,
            xau,
            xag,
            xpt,
            btc,
            eth,
            sol,
            xrp,
            ada,
        } => vec![
            Money::USD(usd),
            Money::CAD(cad),
            Money::EUR(eur),
            Money::GBP(gbp),
            Money::CHF(chf),
            Money::RUB(rub),
            Money::CNY(cny),
            Money::JPY(jpy),
            Money::KRW(krw),
            Money::HKD(hkd),
            Money::IDR(idr),
            Money::MYR(myr),
            Money::SGD(sgd),
            Money::THB(thb),
            Money::SAR(sar),
            Money::AED(aed),
            Money::KWD(kwd),
            Money::INR(inr),
            Money::AUD(aud),
            Money::NZD(nzd),
            Money::XAU(xau),
            Money::XAG(xag),
            Money::XPT(xpt),
            Money::BTC(btc),
            Money::ETH(eth),
            Money::SOL(sol),
            Money::XRP(xrp),
            Money::ADA(ada),
        ],
    };

    let money_variants_count = Money::iter().count();
    let currency_variants_count = Currency::iter().count();

    println!(
        "Money variants: {}, \nCurrency variants: {}, \nret count: {}",
        money_variants_count,
        currency_variants_count,
        ret.len()
    );

    assert_eq!(ret.len(), money_variants_count);
    assert_eq!(ret.len(), currency_variants_count);
    assert_eq!(money_variants_count, currency_variants_count);
}
