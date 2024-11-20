create schema if not exists eth;

create table if not exists eth.blocks (
    id serial primary key,
    block_number NUMERIC unique,
    block_hash BYTEA,
    block_timestamp TIMESTAMP
);

create table if not exists eth.borrow_events (
    id SERIAL PRIMARY KEY,
    block_id INTEGER,
    amount NUMERIC NOT NULL,
    borrow_rate NUMERIC NOT NULL,
    on_behalf_of BYTEA NULL,
    referral_code INTEGER NOT NULL,
    reserve BYTEA NOT NULL,
    "user" BYTEA NOT NULL,
    FOREIGN KEY (block_id) REFERENCES eth.blocks(id) ON DELETE CASCADE
);
create table if not exists eth.repay_events (
    id SERIAL PRIMARY KEY,
    block_id INTEGER,
    amount NUMERIC not null,
    address BYTEA not null,
    use_a_tokens bool not null,
    reserve BYTEA not null,
    "user" BYTEA not null,
    repayer BYTEA not null,
    is_removed bool not null,
    FOREIGN KEY (block_id) REFERENCES eth.blocks(id) ON DELETE CASCADE
);
create table if not exists eth.withdraw_events (
    id integer PRIMARY KEY,
    block_id INTEGER,
    amount NUMERIC not null,
    address BYTEA not null,
    reserve BYTEA not null,
    "user" BYTEA not null,
    to_address BYTEA not null,
    is_removed bool not null,
    FOREIGN KEY (block_id) REFERENCES eth.blocks(id) ON DELETE CASCADE
);
create table if not exists eth.liquidation_events (
    id integer PRIMARY KEY,
    block_id INTEGER,
    address text not null,
    collateral_asset text not null,
    debt_asset text not null,
    debt_to_cover text not null,
    liquidated_collateral_amount text not null,
    liquidator text not null,
    is_removed bool not null,
    "user" text not null,
    FOREIGN KEY (block_id) REFERENCES eth.blocks(id) ON DELETE CASCADE
);