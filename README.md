# Carbon Axelar Execute Relayer

The purpose of this relayer is to facilitate execution to exeternal chain on Carbon-Axelar bridge.
e.g. withdrawals, or any external executions

## How it works:

- Watch and save `Switcheo.carbon.bridge.WithdrawTokenAcknowledgedEvent` from Carbon where event.relayer_deposit_address == `relayer_deposit_address` in config
- Watch and save `ContractCallApproved` event from external chain's Axelar Gateway if the `payload_hash` matches the `WithdrawTokenAcknowledgedEvent` record in the DB
- poll any new event saved,
  - check `is_contract_call_approved` to see if it's already executed
  - check enough relay fees are sent by user 
  - execute to the external blockchain to process the withdrawal or GMP

TODO:
- [ ] whitelist flag for admin relays
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

#### Run migration
```bash
sqlx migrate run
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