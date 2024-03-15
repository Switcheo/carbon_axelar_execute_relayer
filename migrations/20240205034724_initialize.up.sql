-- Add up migration script here
CREATE TABLE payload_acknowledged_events
(
    id               SERIAL PRIMARY KEY,
    bridge_id        TEXT    NOT NULL CHECK (bridge_id <> ''),
    chain_id         TEXT    NOT NULL CHECK (chain_id <> ''),
    payload_type     INTEGER NOT NULL,
    nonce            NUMERIC NOT NULL,
    payload_hash     TEXT    NOT NULL UNIQUE CHECK (payload_hash <> ''),
    payload          TEXT    NOT NULL CHECK (payload <> ''),
    payload_encoding TEXT    NOT NULL CHECK (payload_encoding <> '')
);

CREATE TABLE withdraw_token_confirmed_events
(
    id                      SERIAL PRIMARY KEY,
    coin                    JSONB   NOT NULL,
    connection_id           TEXT    NOT NULL CHECK (connection_id <> ''),
    receiver                TEXT    NOT NULL CHECK (receiver <> ''),
    relay_fee               JSONB   NOT NULL,
    relayer_deposit_address TEXT    NOT NULL CHECK (relayer_deposit_address <> ''),
    sender                  TEXT    NOT NULL CHECK (sender <> ''),
    nonce                   NUMERIC NOT NULL
);

CREATE TABLE contract_call_approved_events
(
    id                 SERIAL PRIMARY KEY,
    blockchain         TEXT    NOT NULL CHECK (blockchain <> ''),
    broadcast_status   TEXT    NOT NULL CHECK (broadcast_status <> ''),
    command_id         TEXT    NOT NULL CHECK (command_id <> ''),
    source_chain       TEXT    NOT NULL CHECK (source_chain <> ''),
    source_address     TEXT    NOT NULL CHECK (source_address <> ''),
    contract_address   TEXT    NOT NULL CHECK (contract_address <> ''),
    payload_hash       TEXT    NOT NULL UNIQUE CHECK (payload_hash <> ''),
    source_tx_hash     TEXT    NOT NULL CHECK (source_tx_hash <> ''),
    source_event_index NUMERIC NOT NULL,
    payload            TEXT    NOT NULL CHECK (payload <> '')
);