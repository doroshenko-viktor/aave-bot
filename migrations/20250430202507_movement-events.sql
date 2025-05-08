DO $$
BEGIN
    IF NOT EXISTS (
        SELECT
            1
        FROM
            pg_type
        WHERE
            typname = 'aave_event_type') THEN
    CREATE TYPE aave_event_type AS ENUM (
        'supply',
        'withdraw',
        'borrow',
        'repay',
        'collateral_transfer',
        'liquidation_repay_debt',
        'liquidation_withdraw_collateral',
);
END IF;
END
$$;

CREATE TABLE IF NOT EXISTS operational.movement_events (
    id serial PRIMARY KEY,
    event_type aave_event_type NOT NULL,
    transaction_hash varchar(64) NOT NULL,
    account_address varchar(42) NOT NULL,
    token_id integer NOT NULL,
    native_token_amount numeric(78, 18) NOT NULL,
    reserve_token_amount numeric(78, 18) NOT NULL,
    FOREIGN KEY (token_id) REFERENCES tokens.tokens (id)
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_movement_events_tx_token ON operational.movement_events (transaction_hash, token_id);

