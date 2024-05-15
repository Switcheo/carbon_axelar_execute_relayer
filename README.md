# Carbon Axelar Execute Relayer

The purpose of this relayer is to facilitate execution to exeternal chain on Carbon-Axelar bridge.
e.g. withdrawals, or any external executions

## How it works:

- Watch and save `Switcheo.carbon.bridge.PayloadAcknowledgedEvent` from Carbon where event.relayer_deposit_address == `relayer_deposit_address` in config
- Watch and save `ContractCallApproved` event from external chain's Axelar Gateway if the `payload_hash` matches the `PayloadAcknowledgedEvent` record in the DB
- poll any new event saved,
  - check `is_contract_call_approved` to see if it's already executed
  - if it is a withdrawal, check enough relay fees are sent by user 
  - execute to the external blockchain to process the withdrawal or GMP

## How it works (new):
- Watch `Switcheo.carbon.bridge.PendingActionEvent` from Carbon
- Check if fees are profitable for relay (See below)
- If profitable, save `Switcheo.carbon.bridge.PendingActionEvent` record in DB with its nonce
- Call StartRelay on Carbon
- Watch for `BridgeRevertedEvent` (delete action and stop processing)
- Watch and save `Switcheo.carbon.bridge.AxelarCallContractEvent` from Carbon where event.nonce matches nonce in DB
- Watch and save `ContractCallApproved` event from external chain's Axelar Gateway if the `payload_hash` matches the `AxelarCallContractEvent` record in the DB
- poll any new event saved,
  - check `is_contract_call_approved` to see if it's already executed
  - execute to the external blockchain to process the withdrawal or GMP

TODO:
- [ ] proper fee conversion
- [ ] rebroadcast from cli

## Setup

#### Install rust/cargo
- https://www.rust-lang.org/tools/install

#### Install Postgres
- mac: `brew install postgresql@15`
- ubuntu: https://www.postgresql.org/download/linux/ubuntu/

#### Install sqlx
```bash
cargo install sqlx-cli --no-default-features --features native-tls,postgres
```

#### Create a `.env` file and add 
```dotenv
# env variable for local development with sqlx (to enable migration etc.)
DATABASE_URL=postgresql://localhost:5432/carbon_axelar_execute_relayer
```

#### Setup Database
```bash
sqlx database setup
```

#### Copy sample config
```bash
cp config.sample.toml config.toml
```

## Usage

#### Run without compiling 
```bash
# run
cargo run -- -vv run

# run with more logs, number of v's:
#        0 => Level::ERROR,
#        1 => Level::WARN,
#        2 => Level::INFO,
#        3 => Level::DEBUG,
#        >3 => Level::TRACE,
cargo run -- -vvvv run

# run with a different config file
cargo run -- -vv run --config your_config.toml
```

#### Compile binary
```bash
# compiles to ./target/debug/carbon_axelar_execute_relayer
cargo build
# compiles to ./target/release/carbon_axelar_execute_relayer
cargo build --release
```

#### Database
```bash
# reset database
sqlx database reset
# migrate
sqlx migrate run
# rollback
sqlx migrate revert

# others, use help for more info
sqlx --help
sqlx database --help
sqlx <command> --help
```

## Stuck Transactions

Sometimes transactions get stuck at various points either when a relayer is down or an RPC or WS endpoint is down.

The way to resolve this is to manually call the cli to save the tx since it is missed


### Commands

#### Resync

```bash
# resync from carbon's start block height to end block height to populate missed events so that they can be relayed
cargo run -- -vv sync-from 318371 318490

```

#### Start Relay

```bash
# start a relay on a nonce
cargo run -- -vv start-relay 1
```