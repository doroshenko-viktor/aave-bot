-- These are configuration tables which are read once on startup and then should be cached in memory.
-- No special id field for these tables because here readability is more valuable.
-- These tables are populated by sql scripts on startup or after migrations, so populating sql scripts readability is preferred.
CREATE TABLE IF NOT EXISTS tokens.tokens (
    id serial PRIMARY KEY,
    address bytea UNIQUE NOT NULL,
    "name" varchar(50),
    symbol varchar(10) UNIQUE NOT NULL,
    decimals integer NOT NULL,
    total_supply numeric(78, 0) NOT NULL,
    a_token_address bytea NOT NULL,
    stable_debt_token_address bytea NOT NULL,
    variable_debt_token_address bytea NOT NULL
);

CREATE TABLE IF NOT EXISTS tokens.token_config (
    id serial PRIMARY KEY,
    symbol varchar(6) NOT NULL UNIQUE,
    is_active boolean NOT NULL,
    "comment" text NULL,
    token_id int NOT NULL,
    FOREIGN KEY (token_id) REFERENCES tokens.tokens (id)
);


-- CREATE TABLE IF NOT EXISTS price_feed.tickers (
--     token_1 varchar(6) NOT NULL,
--     token_2 varchar(6) NOT NULL,
--     bybit_repr varchar(15) NOT NULL,
--     chainlink_repr varchar(15) NOT NULL,
--     pyth_repr varchar(15) NOT NULL,
--     PRIMARY KEY (token_1, token_2),
--     FOREIGN key (token_1) REFERENCES tokens.token_details (symbol),
--     FOREIGN key (token_2) REFERENCES tokens.token_details (symbol)
-- );
