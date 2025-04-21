# pfm (WIP)

## A Portfolio Manager

Project layout:
- pfm-core: contains core logics, rules and core data(e.g. prices) of forex(fiats, precious metals, and crypto), stocks, etc.
  - forex: contains pricing for fiats, precious metals, and cryptos. Currently support: USD,CAD,EUR,GBP,CHF,RUB,CNY,JPY,KRW,HKD,IDR,MYR,SGD,THB,SAR,AED,KWD,INR,AUD,NZD,XAU,XAG,XPT,BTC,ETH,SOL,XRP,ADA.
  - pm: contains data for precious metals, such units(grams, ounces, and kilograms), purity, and prices.
  - ...
- pfm-http: serve pfm APIs. Currently endpoints supported:
  - forex(LIVE): conversion, rates and timeseries APIs between above supported currencies.
  - ...
- pfm-cron(LIVE): periodic update of core data(e.g. prices)
- pfm-zakat: manage zakat such nishab calculation, payment due date, using updated price data.
- pfm-cli: cli app for managing portfolio data. (TODO)
- pfm-web: web interface for managing portfolio data. (TODO)
- pfm-rag: use pfm data as context for LLM. (TODO)

