CREATE SCHEMA if NOT EXISTS operational;

CREATE TABLE IF NOT EXISTS operational.accounts (
    id serial PRIMARY KEY,
    address varchar(42) NOT NULL UNIQUE
);

CREATE TABLE operational.account_collaterals (
    id serial PRIMARY KEY,
    account_address varchar(42) NOT NULL,
    token_symbol varchar(6) NOT NULL,
    amount numeric(78, 0) NOT NULL,
    FOREIGN key (account_address) REFERENCES operational.accounts (address),
    UNIQUE (account_address, token_symbol)
);

CREATE TABLE operational.account_debts (
    id serial PRIMARY KEY,
    account_address varchar(42) NOT NULL,
    token_symbol varchar(6) NOT NULL,
    amount numeric(78, 0) NOT NULL,
    FOREIGN key (account_address) REFERENCES operational.accounts (address),
    UNIQUE (account_address, token_symbol)
);
