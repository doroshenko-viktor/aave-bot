DELETE FROM config.tickers
WHERE
    1=1;

--
INSERT INTO
    config.tickers (
        token_1,
        token_2,
        bybit_repr,
        chainlink_repr,
        pyth_repr
    )
VALUES
    ("BTC", "USDT", "BTCUSDT"),
    ("BTC", "USDC", "BTCUSDC"),
    ("ETH", "USDT", "ETHUSDT"),
    ("ETH", "USDC", "ETHUSDC");

--