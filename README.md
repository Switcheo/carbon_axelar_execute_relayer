# Carbon Axelar Execute Relayer

[![Rust Build](https://github.com/Switcheo/carbon_axelar_execute_relayer/actions/workflows/rust.yml/badge.svg)](https://github.com/Switcheo/carbon_axelar_execute_relayer/actions/workflows/rust.yml)
[![Rust](https://img.shields.io/badge/rust-stable-orange.svg)](https://www.rust-lang.org/)

The purpose of this relayer is to facilitate execution to external chain on Carbon-Axelar bridge.
e.g. withdrawals, or any external executions

## How it works:
- Watch `Switcheo.carbon.bridge.NewPendingActionEvent` from Carbon
- Check if fees are profitable for relay (See below)
- If profitable, save `Switcheo.carbon.bridge.NewPendingActionEvent` record in DB with its nonce
- Call StartRelay on Carbon
- Watch for `BridgeRevertedEvent` (delete action and stop processing)
- Watch and save `Switcheo.carbon.bridge.AxelarCallContractEvent` from Carbon where event.nonce matches nonce in DB
- Watch and save `ContractCallApproved` event from external chain's Axelar Gateway if the `payload_hash` matches the `AxelarCallContractEvent` record in the DB
- poll any new event saved,
  - check `is_contract_call_approved` to see if it's already executed
  - execute to the external blockchain to process the withdrawal or GMP

Note: relayer needs to be whitelisted on carbon

## Linux dependencies (only for linux)
```
# install libssl-dev pkg-config
sudo apt install libssl-dev pkg-config

# install buf
# Substitute BIN for your bin directory.
# Substitute VERSION for the current released version.
BIN="/usr/local/bin" && \
VERSION="1.34.0" && \
curl -sSL \
"https://github.com/bufbuild/buf/releases/download/v${VERSION}/buf-$(uname -s)-$(uname -m)" \
-o "${BIN}/buf" && \
chmod +x "${BIN}/buf"

```

## Setup

#### Install rust/cargo
follow installation:
- https://www.rust-lang.org/tools/install

then use 
```bash
rustup install 1.85.1
rustup default 1.85.1
```

#### Linux preinstalls
```bash
# pre: only if on linux
sudo apt update
sudo apt install build-essential pkg-config libssl-dev -y
```

#### Install Postgres
- mac: `brew install postgresql@15`
- ubuntu: https://www.postgresql.org/download/linux/ubuntu/

#### Install sqlx
```bash
# install sqlx
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

## Usage (Binary - Ubuntu)

```bash
# download and setup
VERSION=v1.0.5
wget https://github.com/Switcheo/carbon_axelar_execute_relayer/releases/download/$VERSION/carbon_axelar_execute_relayer-x86_64-unknown-linux-gnu.tar.gz
tar -xvzf carbon_axelar_execute_relayer-x86_64-unknown-linux-gnu.tar.gz
chmod +x carbon_axelar_execute_relayer

# run
./carbon_axelar_execute_relayer -vv run
```

## Usage (Dev)

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
cargo run -- --config your_config.toml -vv run
```

#### Compile binary
```bash
# prepares sqlx for compiling
cargo sqlx prepare -- --bin carbon_axelar_execute_relayer
# compiles to ./target/debug/carbon_axelar_execute_relayer
cargo build
# compiles to ./target/release/carbon_axelar_execute_relayer
cargo build --release
```

release
```bash
# optionally, add release tag to trigger compilation on github
git tag v1.0.0
git push origin v1.0.0

# INFO: Deleting a tag to re-release
git tag -d v1.0.0
git push origin :refs/tags/v1.0.0
git tag v1.0.0
git push origin v1.0.0
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

```bash
# using hex payload
cargo run -- -vv execute-contract-call-approved mantle-testnet 0xcdebbc1eb3895b9d709c0e4f098b8be3ea600fa3c894b0a8c693e012ee501720 0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000240000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000006000000000000000000000000040eeba3ba9b2afef980079a841cd4693e3c835c0000000000000000000000000ad90498006ecb49a3bfa145aa99cb08573f924530000000000000000000000000000000000000000000000056bc75e2d63100000
# using base64 payload, with config file config.develop.toml
cargo run -- --config config.develop.toml -vv execute-contract-call-approved mantle-testnet 0xcdebbc1eb3895b9d709c0e4f098b8be3ea600fa3c894b0a8c693e012ee501720 AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAACAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA7AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAGAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAYAAAAAAAAAAAAAAAAOc69adqgRoWpA/5PqFt/qEMSToOAAAAAAAAAAAAAAAAp2UQC2kn1Lb2oszyaBeY1T4f0QEAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAQ==
```


### Commands

#### Resync

```bash
# resync from carbon's start block height to end block height to populate missed events so that they can be relayed
cargo run -- -vv sync-from 788086 788099

# resync from carbon's start block height to end block height to populate missed events so that they can be relayed
# specify which block to search for evm events
cargo run -- -vv sync-from 788086 788099

```

#### Start Relay

```bash
# start a relay on a nonce
cargo run -- -vv start-relay 1
```

#### Expire Pending Actions

```bash
# expires pending actions by their nonces
cargo run -- -vv expire-pending-actions 1,2,3
```

## Generating protos

**Pre-requisite: install `buf` cli on your computer https://buf.build/docs/cli/installation/**

```bash
# Note: `cd` into `proto` folder first before running buf
cd proto

# update the proto dependencies in buf.yaml
buf dep update

# generate
buf generate
```
