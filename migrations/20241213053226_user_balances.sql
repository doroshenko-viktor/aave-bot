CREATE SCHEMA IF NOT EXISTS operational;

CREATE TABLE IF NOT EXISTS operational.accounts (
    id serial PRIMARY KEY,
    address varchar(42) NOT NULL UNIQUE,
    comment text NULL,
);

CREATE TABLE operational.account_collaterals (
    id serial PRIMARY KEY,
    account_address varchar(42) NOT NULL,
    token_symbol varchar(6) NOT NULL,
    amount numeric(78, 0) NOT NULL,
    a_token_amount numeric(78, 0) NOT NULL,
    FOREIGN KEY (account_address) REFERENCES operational.accounts (address),
    UNIQUE (account_address, token_symbol)
);

CREATE TABLE operational.account_debts (
    id serial PRIMARY KEY,
    account_address varchar(42) NOT NULL,
    token_symbol varchar(6) NOT NULL,
    amount numeric(78, 0) NOT NULL,
    debt_token_amount numeric(78, 0) NOT NULL,
    FOREIGN KEY (account_address) REFERENCES operational.accounts (address),
    UNIQUE (account_address, token_symbol)
);

