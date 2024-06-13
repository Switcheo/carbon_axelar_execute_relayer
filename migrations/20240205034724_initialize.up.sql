-- Add up migration script here
CREATE TABLE pending_action_events
(
    id                      SERIAL      PRIMARY KEY,
    connection_id           TEXT        NOT NULL CHECK (connection_id <> ''),
    bridge_id               TEXT        NOT NULL CHECK (bridge_id <> ''),
    chain_id                TEXT        NOT NULL CHECK (chain_id <> ''),
    nonce                   NUMERIC     NOT NULL UNIQUE,
    pending_action_type     INTEGER     NOT NULL,
    retry_count             INTEGER     NOT NULL,
    relay_details           JSONB       NOT NULL
);

CREATE TABLE axelar_call_contract_events
(
    id               SERIAL PRIMARY KEY,
    nonce            NUMERIC NOT NULL,
    payload_hash     TEXT    NOT NULL UNIQUE CHECK (payload_hash <> ''),
    payload          TEXT    NOT NULL CHECK (payload <> ''),
    payload_encoding TEXT    NOT NULL CHECK (payload_encoding <> '')
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