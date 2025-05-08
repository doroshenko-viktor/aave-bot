SELECT
    transaction_hash,
    block_number,
    log_number,
    event_type_id,
    body
FROM
    blockchain.events
    --        where block_number >= $1 and block_number <= $2 and
WHERE (body ->> 'user' = '0x5115563fe64378f3a645c97e24c4c8d640a04c67'
    OR body ->> 'on_behalf_of' = '0x5115563fe64378f3a645c97e24c4c8d640a04c67'
    OR body ->> 'from' = '0x5115563fe64378f3a645c97e24c4c8d640a04c67'
    OR body ->> 'to' = '0x5115563fe64378f3a645c97e24c4c8d640a04c67')
ORDER BY
    block_number,
    log_number ASC;

