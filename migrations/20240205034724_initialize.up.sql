-- Add up migration script here
CREATE TABLE payload_acknowledged_events
(
    id                      SERIAL PRIMARY KEY,
    payload_type            NUMERIC NOT NULL,
    nonce                   NUMERIC NOT NULL,
    payload_hash            TEXT  NOT NULL UNIQUE,
    payload                 TEXT  NOT NULL,
    payload_encoding        TEXT  NOT NULL
);

CREATE TABLE withdraw_token_acknowledged_events
(
    id                      SERIAL PRIMARY KEY,
    coin                    JSONB NOT NULL,
    connection_id           TEXT  NOT NULL,
    receiver                TEXT  NOT NULL,
    relay_fee               JSONB NOT NULL,
    relayer_deposit_address TEXT  NOT NULL,
    sender                  TEXT  NOT NULL,
    payload_hash            TEXT  NOT NULL UNIQUE,
    payload                 TEXT  NOT NULL
);

CREATE TABLE contract_call_approved_events
(
    id                 SERIAL PRIMARY KEY,
    blockchain         TEXT    NOT NULL,
    broadcast_status   TEXT    NOT NULL,
    command_id         TEXT    NOT NULL,
    source_chain       TEXT    NOT NULL,
    source_address     TEXT    NOT NULL,
    contract_address   TEXT    NOT NULL,
    payload_hash       TEXT    NOT NULL UNIQUE,
    source_tx_hash     TEXT    NOT NULL,
    source_event_index NUMERIC NOT NULL,
    payload            TEXT    NOT NULL
);