-- These are configuration tables which are read once on startup and then should be cached in memory.
-- No special id field for these tables because here readability is more valuable.
-- These tables are populated by sql scripts on startup or after migrations, so populating sql scripts readability is preferred.
CREATE TABLE IF NOT EXISTS tokens.token_details (
    id serial PRIMARY KEY,
    symbol varchar(6) NOT NULL UNIQUE,
    is_active boolean NOT NULL,
    comment text NULL
);

CREATE TABLE IF NOT EXISTS tokens.onchain_token_data (
    id serial PRIMARY KEY,
    token_id int NOT NULL,
    address bytea UNIQUE NOT NULL,
    name varchar(50),
    symbol varchar(10) NOT NULL,
    decimals integer NOT NULL,
    total_supply numeric NOT NULL,
    FOREIGN key (token_id) REFERENCES tokens.token_details (id)
);

CREATE TABLE IF NOT EXISTS price_feed.tickers (
    token_1 varchar(6) NOT NULL,
    token_2 varchar(6) NOT NULL,
    bybit_repr varchar(15) NOT NULL,
    chainlink_repr varchar(15) NOT NULL,
    pyth_repr varchar(15) NOT NULL,
    PRIMARY KEY (token_1, token_2),
    FOREIGN key (token_1) REFERENCES tokens.token_details (symbol),
    FOREIGN key (token_2) REFERENCES tokens.token_details (symbol)
);
