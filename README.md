# Tracker Crypto — TRON USDT Wallet Tracker (Rust)

A high-performance batch processing pipeline that tracks USDT (TRC-20) transactions on the TRON network. Built with Rust for speed and reliability, it enriches wallet data using TronScan and Arkham Intelligence APIs, filters out exchange wallets, and stores everything in a Turso/libSQL database.

## Architecture

```
┌──────────────┐     ┌──────────────────┐     ┌──────────────┐
│  TronScan    │────▶│  tracker_crypto  │────▶│   Turso /    │
│  API         │     │  (Rust binary)   │     │   SQLite DB  │
└──────────────┘     └────────┬─────────┘     └──────────────┘
                              │
                        ┌─────▼──────┐
                        │   Arkham   │
                        │ Intelligence│
                        └────────────┘
```

**Pipeline phases:**
1. **Seed** — If no accounts exist, processes 5 hardcoded TRON wallets
2. **Track** — Continuously processes wallets from the `accounts` table (Arkham-enriched but not yet tracked)
3. **Enrich** — Identifies exchanges, contracts, and labels via Arkham Intelligence
4. **Collect** — Fetches total USD value for each wallet

**Control mechanism:** File-based kill switches (`pull_wallets`, `pull_arkm`, `pull_amount`) — delete the file to stop the corresponding loop.

## Prerequisites

- Rust >= 1.75 ([Install Rust](https://rustup.rs/))
- Cargo (included with Rust)
- [Turso CLI](https://docs.turso.tech/cli) (for local development)
- API keys:
  - **TronScan** — Get from [TronScan API](https://apilist.tronscanapi.com/)
  - **Arkham Intelligence** — Get from [Arkham](https://platform.arkhamintelligence.com/)

## Setup

```bash
# Clone the repository
git clone <repo-url>
cd tracker_crypto

# Build the project
cargo build --release

# Configure environment variables
cat << EOF > .env
TURSO_URL=http://127.0.0.1:8080
# TURSO_AUTH_TOKEN=
TRON_TOKEN=your_tronscan_api_key
ARKM_API_KEY=your_arkham_api_key
EOF

# Start local Turso database
turso dev --db-file wallets01.db
```

Migrations run automatically on startup via Diesel.

## Usage

### Running the Pipeline

```bash
# Run in foreground
cargo run --release

# Or run the compiled binary directly
./target/release/tracker_crypto
```

### Process Management

For production, use a process manager:

```bash
# Using PM2
pm2 start ./target/release/tracker_crypto --name tracker
pm2 stop tracker
pm2 logs tracker

# Or using systemd (see Deployment section)
```

### Build Commands

| Command | Description |
|---------|-------------|
| `cargo build` | Build in debug mode |
| `cargo build --release` | Build in release mode (optimized) |
| `cargo run` | Build and run |
| `cargo run --release` | Build and run (optimized) |
| `cargo check` | Check for compilation errors |
| `cargo clippy` | Run linter |
| `cargo fmt` | Format code |
| `cargo test` | Run tests |

### Kill Switches

Create/delete files to control which loops run:

```bash
# Stop wallet tracking loop
rm pull_wallets

# Stop Arkham enrichment loop
rm pull_arkm

# Stop amount collection loop
rm pull_amount
```

## External APIs

| API | Endpoint | Rate Limit | Purpose |
|-----|----------|------------|---------|
| TronScan | `GET /api/transfer/trc20` | 300ms min | Fetch USDT transfer history (paginated, 50/page) |
| TronScan | `GET /api/accountv2` | 300ms min | Fetch account metadata |
| TronScan | `GET /api/account/wallet` | 300ms min | Fetch wallet USD value |
| TronScan | `GET /api/deep/account/holderToken/basicInfo/trc20/transfer` | 300ms min | Fetch USDT transfer summary |
| Arkham | `POST /intelligence/address_enriched/batch` | 1100ms min | Batch wallet enrichment |

Rate limiting is handled using `tokio::time` or the [`governor`](https://crates.io/crates/governor) crate.

## Database Schema

Managed by Diesel migrations. Schema is defined in `migrations/2024-01-01-000000_initial_schema/up.sql`.

### `accounts`

Stores wallet addresses and their metadata.

| Column | Type | Description |
|--------|------|-------------|
| `id` | INTEGER (PK) | Auto-increment ID |
| `wallet` | TEXT (UNIQUE) | TRON wallet address |
| `exchange_name` | TEXT | Exchange label from TronScan |
| `is_exchange` | INTEGER (0/1) | Exchange flag from TronScan |
| `is_contract` | INTEGER (0/1) | Contract flag from TronScan |
| `is_tracked` | INTEGER/NULL | 1 if TronScan tracking completed |
| `transferIn` | INTEGER | Inbound transfer count |
| `transferOut` | INTEGER | Outbound transfer count |
| `transactionsTron` | INTEGER | Total TRON transaction count |
| `balanceTron` | INTEGER | TRON balance |
| `deep` | INTEGER | Depth level |
| `payload_tronscan` | TEXT | Full TronScan API response (JSON) |
| `is_exchange_arkm` | INTEGER (0/1) | Exchange flag from Arkham |
| `is_contract_arkm` | INTEGER (0/1) | Contract flag from Arkham |
| `is_tracked_arkm` | INTEGER/NULL | 1 if Arkham enrichment completed |
| `payload_arkm` | TEXT | Full Arkham API response (JSON) |
| `arkham_label` | TEXT | Label from Arkham |
| `populated_tags` | TEXT | Arkham tags array (JSON) |
| `mandatory_scan` | INTEGER | Mandatory scan flag |
| `observations` | TEXT | Notes/observations |
| `is_receiver` | INTEGER (0/1) | 1 if wallet received funds |
| `created_at` | INTEGER | Creation timestamp (epoch seconds) |
| `updated_at` | INTEGER | Last update timestamp (epoch seconds, auto-updated via trigger) |
| `is_amount_collected` | INTEGER (0/1) | Whether USD amount was fetched |
| `total_usd_amount` | REAL | Total USD value of wallet |

### `transactions`

Stores USDT transfer records.

| Column | Type | Description |
|--------|------|-------------|
| `hash_tx` | TEXT (PK) | Transaction hash |
| `amount` | TEXT | USDT transfer amount |
| `status` | INTEGER | Transaction status |
| `approval_amount` | TEXT | Approval amount |
| `block_timestamp` | INTEGER | Block timestamp |
| `block` | INTEGER | Block number |
| `wallet_from` | TEXT | Sender address |
| `wallet_to` | TEXT | Receiver address |
| `wallet_from_id` | INTEGER (FK) | References `accounts.id` |
| `wallet_to_id` | INTEGER (FK) | References `accounts.id` |
| `confirmed` | INTEGER | Confirmation status |
| `contract_type` | TEXT | Contract type |
| `contractType` | INTEGER | Contract type (alternate) |
| `revert` | INTEGER | Revert flag |
| `contract_ret` | TEXT | Contract return status |
| `event_type` | TEXT | Event type |
| `issue_address` | TEXT | Token issuer address |
| `decimals` | INTEGER | Token decimals |
| `exchange_from` | TEXT | Exchange name of sender |
| `exchange_to` | TEXT | Exchange name of receiver |
| `is_sent_to_exchange` | INTEGER | Whether sent to exchange |
| `direction` | INTEGER | Transfer direction |
| `updated_at` | INTEGER | Last update timestamp (epoch seconds, auto-updated via trigger) |

### `mv_user_wallets_groups_export`

Legacy table for checking previously processed wallets.

| Column | Type | Description |
|--------|------|-------------|
| `cc_addresses` | TEXT | Wallet address |

## Project Structure

```
tracker_crypto/
├── README.md
├── Cargo.toml
├── diesel.toml                 # Diesel CLI configuration
├── migrations/
│   └── 2024-01-01-000000_initial_schema/
│       ├── up.sql              # Schema creation
│       └── down.sql            # Schema rollback
├── src/
│   ├── main.rs                 # Entry point, pipeline orchestration
│   ├── schema.rs               # Auto-generated Diesel schema
│   ├── models.rs               # Diesel ORM models + API response structs
│   ├── db.rs                   # Database operations (Diesel)
│   ├── tracker.rs              # Core tracking logic
│   ├── tronscan.rs             # TronScan API client
│   ├── arkham.rs               # Arkham Intelligence client
│   ├── config.rs               # Configuration (.env loading)
│   ├── arkham_tests.rs         # Arkham parsing tests
│   └── db_tests.rs             # Database operation tests
├── pull_wallets                # Kill switch: wallet tracking
├── pull_arkm                   # Kill switch: Arkham enrichment
├── pull_amount                 # Kill switch: amount collection
└── nodejs/                     # Reference Node.js implementation
```

## Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `tokio` | 1.x | Async runtime |
| `reqwest` | 0.12.x | HTTP client |
| `diesel` | 2.3.x | ORM and query builder |
| `diesel-async` | 0.7.x | Async Diesel connections |
| `diesel_migrations` | 2.3.x | Database migration management |
| `serde` | 1.x | JSON serialization/deserialization |
| `serde_json` | 1.x | JSON handling |
| `dotenvy` | 0.15.x | Environment variable loading |
| `governor` | 0.8.x | Rate limiting |
| `anyhow` | 1.x | Error handling |
| `tracing` | 0.1.x | Logging/tracing |
| `tracing-subscriber` | 0.3.x | Log formatting |

## Deployment

### Binary Deployment

```bash
# Build for production
cargo build --release

# The binary will be at:
# ./target/release/tracker_crypto

# Copy to server
scp target/release/tracker_crypto user@server:/opt/tracker/
```

### Systemd Service

```ini
# /etc/systemd/system/tracker.service
[Unit]
Description=TRON USDT Wallet Tracker
After=network.target

[Service]
Type=simple
User=tracker
WorkingDirectory=/opt/tracker
ExecStart=/opt/tracker/tracker_crypto
Restart=always
RestartSec=10
EnvironmentFile=/opt/tracker/.env

[Install]
WantedBy=multi-user.target
```

```bash
sudo systemctl enable tracker
sudo systemctl start tracker
sudo journalctl -u tracker -f
```

## License

ISC
