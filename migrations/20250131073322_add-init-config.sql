-- Insert values
INSERT INTO
    config.cache (key, value)
VALUES
    ('last_user_balances_block', '0')
ON CONFLICT (key) DO UPDATE SET value=excluded.value;

INSERT INTO
    config.cache (key, value) -- VALUES ('last_handled_block', '16496939') ON CONFLICT (key) DO
VALUES
    ('last_downloaded_block', '0')
ON CONFLICT (key) DO UPDATE
SET
    value=excluded.value;
