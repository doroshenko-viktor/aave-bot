create table if not exists blockchain.tokens (
    id serial primary key,
    address BYTEA unique not null,
    name varchar(50),
    symbol varchar(10) not null,
    decimals integer not null,
    total_supply NUMERIC not null
)