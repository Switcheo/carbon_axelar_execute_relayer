# Carbon Axelar Execute Relayer

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
# Postgres
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
cargo run -- run

# run with a different config file
cargo run -- run --config your_config.toml
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