create schema if not exists blockchain;
create schema if not exists config;
create table if not exists config.cache (
    id serial primary key,
    key varchar(50),
    value varchar,
    constraint idx_key_unique unique (key)
);
create table if not exists blockchain.blocks (
    id serial primary key,
    block_number NUMERIC(78, 18) unique,
    block_hash BYTEA,
    block_timestamp TIMESTAMP
);
create table if not exists blockchain.supply_events (
    id SERIAL PRIMARY KEY,
    block_number INTEGER NOT NULL,
    log_id INTEGER NOT NULL,
    reserve BYTEA NOT NULL,
    "user" BYTEA NOT NULL,
    on_behalf_of BYTEA NULL,
    amount NUMERIC(78, 18) NOT NULL,
    referral_code INTEGER NOT NULL,
    UNIQUE (block_number, log_id)
);
create table if not exists blockchain.borrow_events (
    id SERIAL PRIMARY KEY,
    block_number INTEGER NOT NULL,
    log_id INTEGER NOT NULL,
    amount NUMERIC(78, 18) NOT NULL,
    borrow_rate NUMERIC(78, 18) NOT NULL,
    interest_rate_mode INTEGER NOT NULL,
    on_behalf_of BYTEA NULL,
    referral_code INTEGER NOT NULL,
    reserve BYTEA NOT NULL,
    "user" BYTEA NOT NULL,
    -- FOREIGN KEY (block_number) REFERENCES eth.blocks(id) ON DELETE CASCADE,
    UNIQUE (block_number, log_id)
);
create table if not exists blockchain.repay_events (
    id SERIAL PRIMARY KEY,
    block_number INTEGER NOT NULL,
    log_id INTEGER NOT NULL,
    amount NUMERIC(78, 18) not null,
    reserve BYTEA not null,
    "user" BYTEA not null,
    repayer BYTEA not null,
    use_a_tokens bool not null,
    -- FOREIGN KEY (block_number) REFERENCES eth.blocks(id) ON DELETE CASCADE,
    UNIQUE (block_number, log_id)
);
create table if not exists blockchain.withdraw_events (
    id SERIAL PRIMARY KEY,
    block_number INTEGER NOT NULL,
    log_id INTEGER NOT NULL,
    amount NUMERIC(78, 18) not null,
    address BYTEA not null,
    reserve BYTEA not null,
    "user" BYTEA not null,
    "to" BYTEA not null,
    -- FOREIGN KEY (block_number) REFERENCES eth.blocks(id) ON DELETE CASCADE,
    UNIQUE (block_number, log_id)
);
create table if not exists blockchain.liquidation_events (
    id SERIAL PRIMARY KEY,
    block_number INTEGER NOT NULL,
    log_id INTEGER NOT NULL,
    collateral_asset text not null,
    debt_asset text not null,
    debt_to_cover text not null,
    liquidated_collateral_amount text not null,
    liquidator text not null,
    "user" text not null,
    receive_a_token bool not null,
    -- FOREIGN KEY (block_number) REFERENCES eth.blocks(id) ON DELETE CASCADE,
    UNIQUE (block_number, log_id)
);