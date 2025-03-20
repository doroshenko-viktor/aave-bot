CREATE SCHEMA if NOT EXISTS blockchain;

CREATE SCHEMA if NOT EXISTS tokens;

CREATE SCHEMA if NOT EXISTS config;

CREATE SCHEMA if NOT EXISTS logs;

CREATE SCHEMA if NOT EXISTS price_feed;

--
--CREATE TABLES
--
CREATE TABLE IF NOT EXISTS config.cache (
    id serial PRIMARY KEY,
    key varchar(50),
    value varchar,
    CONSTRAINT idx_key_unique UNIQUE (key)
);

CREATE TABLE IF NOT EXISTS blockchain.blocks (
    id serial PRIMARY KEY,
    block_number numeric(78, 18) UNIQUE,
    block_hash bytea,
    block_timestamp timestamp
);

CREATE TABLE IF NOT EXISTS blockchain.events (
    id serial PRIMARY KEY,
    block_number integer NOT NULL,
    log_number integer NOT NULL,
    event_type_id int NOT NULL,
    body jsonb NOT NULL,
    FOREIGN key (block_number) REFERENCES blockchain.blocks (id) ON DELETE cascade,
    UNIQUE (block_number, log_number)
);

-- CREATE INDEX idx_block_number ON blockchain.events (block_number);
--
-- CREATE TABLE IF NOT EXISTS blockchain.supply_events (
--     id SERIAL PRIMARY KEY,
--     block_number INTEGER NOT NULL,
--     log_id INTEGER NOT NULL,
--     reserve BYTEA NOT NULL,
--     "user" BYTEA NOT NULL,
--     on_behalf_of BYTEA NOT NULL,
--     amount NUMERIC(78, 18) NOT NULL,
--     referral_code INTEGER NOT NULL,
--     UNIQUE (block_number, log_id)
-- );
-- CREATE TABLE IF NOT EXISTS blockchain.borrow_events (
--     id SERIAL PRIMARY KEY,
--     block_number INTEGER NOT NULL,
--     log_id INTEGER NOT NULL,
--     amount NUMERIC(78, 18) NOT NULL,
--     borrow_rate NUMERIC(78, 18) NOT NULL,
--     interest_rate_mode INTEGER NOT NULL,
--     on_behalf_of BYTEA NOT NULL,
--     referral_code INTEGER NOT NULL,
--     reserve BYTEA NOT NULL,
--     "user" BYTEA NOT NULL,
--     -- FOREIGN KEY (block_number) REFERENCES eth.blocks(id) ON DELETE CASCADE,
--     UNIQUE (block_number, log_id)
-- );
-- CREATE TABLE IF NOT EXISTS blockchain.repay_events (
--     id SERIAL PRIMARY KEY,
--     block_number INTEGER NOT NULL,
--     log_id INTEGER NOT NULL,
--     amount NUMERIC(78, 18) NOT NULL,
--     reserve BYTEA NOT NULL,
--     "user" BYTEA NOT NULL,
--     repayer BYTEA NOT NULL,
--     use_a_tokens bool NOT NULL,
--     -- FOREIGN KEY (block_number) REFERENCES eth.blocks(id) ON DELETE CASCADE,
--     UNIQUE (block_number, log_id)
-- );
-- CREATE TABLE IF NOT EXISTS blockchain.withdraw_events (
--     id SERIAL PRIMARY KEY,
--     block_number INTEGER NOT NULL,
--     log_id INTEGER NOT NULL,
--     amount NUMERIC(78, 18) NOT NULL,
--     address BYTEA NOT NULL,
--     reserve BYTEA NOT NULL,
--     "user" BYTEA NOT NULL,
--     "to" BYTEA NOT NULL,
--     -- FOREIGN KEY (block_number) REFERENCES eth.blocks(id) ON DELETE CASCADE,
--     UNIQUE (block_number, log_id)
-- );
-- CREATE TABLE IF NOT EXISTS blockchain.liquidation_events (
--     id SERIAL PRIMARY KEY,
--     block_number INTEGER NOT NULL,
--     log_id INTEGER NOT NULL,
--     collateral_asset BYTEA NOT NULL,
--     debt_asset BYTEA NOT NULL,
--     debt_to_cover NUMERIC(78, 18) NOT NULL,
--     liquidated_collateral_amount NUMERIC(78, 18) NOT NULL,
--     liquidator BYTEA NOT NULL,
--     "user" BYTEA NOT NULL,
--     receive_a_token bool NOT NULL,
--     -- FOREIGN KEY (block_number) REFERENCES eth.blocks(id) ON DELETE CASCADE,
--     UNIQUE (block_number, log_id)
-- );
-- CREATE INDEX idx_block_number ON blockchain.liquidation_events(block_number);
CREATE TABLE IF NOT EXISTS logs.errors (
    id serial PRIMARY KEY,
    "timestamp" timestamp,
    message text
);
