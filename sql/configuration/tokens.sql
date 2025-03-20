DELETE FROM config.active_tokens
WHERE
    1=1;

--
INSERT INTO
    config.active_tokens (token, is_active)
VALUES
    ("BTC", TRUE),
    ("ETH", TRUE),
    ("USDT", TRUE),
    ("USDC", TRUE);

--
