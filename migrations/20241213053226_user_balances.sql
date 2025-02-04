CREATE SCHEMA if NOT EXISTS operational;

CREATE TABLE IF NOT EXISTS operational.accounts (
    id serial PRIMARY KEY,
    address bytea NOT NULL,
    UNIQUE (address)
);

CREATE TABLE operational.account_collaterals (
    id serial PRIMARY KEY,
    account_id INTEGER,
    asset_id INTEGER,
    amount NUMERIC(78, 18),
    FOREIGN key (account_id) REFERENCES operational.accounts (id),
    UNIQUE (account_id, asset_id)
);

CREATE TABLE operational.account_debts (
    id serial PRIMARY KEY,
    account_id INTEGER,
    asset_id INTEGER,
    amount NUMERIC(78, 18),
    FOREIGN key (account_id) REFERENCES operational.accounts (id),
    UNIQUE (account_id, asset_id)
);
