# AGENTS.md — Tracker Crypto Development Guide

## Overview

A Rust batch processing pipeline that tracks USDT (TRC-20) transactions on the TRON network. It enriches wallet data using TronScan and Arkham Intelligence APIs, filters out exchange wallets, and stores everything in a Turso database with cloud sync.

## Architecture

```
┌──────────────┐     ┌──────────────────┐     ┌──────────────┐
│  TronScan    │────▶│  tracker_crypto  │────▶│   Turso Cloud│
│  API         │     │  (Rust binary)   │     │  (serverless)│
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

## Tech Stack

| Component | Technology | Version |
|-----------|-----------|---------|
| Language | Rust | >= 1.95 |
| Async Runtime | tokio | 1.x |
| HTTP Client | reqwest | 0.12.x |
| ORM | toasty | 0.11.x |
| Database | Turso (serverless) | via toasty-driver-turso 0.11.x |
| Serialization | serde + serde_json | 1.x |
| Config | dotenvy | 0.15.x |
| Rate Limiting | governor | 0.8.x |
| Error Handling | anyhow | 1.x |
| Logging | tracing + tracing-subscriber | 0.1.x / 0.3.x |

## Project Structure

```
tracker_crypto/
├── README.md
├── Cargo.toml
├── src/
│   ├── main.rs                 # Entry point, pipeline orchestration
│   ├── schema.rs               # Toasty ORM model definitions
│   ├── db.rs                   # Database operations (Toasty + raw SQL)
│   ├── models.rs               # API response structs + helpers
│   ├── tracker.rs              # Core tracking logic
│   ├── tronscan.rs             # TronScan API client
│   ├── arkham.rs               # Arkham Intelligence client
│   ├── config.rs               # Configuration (.env loading)
│   ├── arkham_tests.rs         # Arkham parsing tests
│   └── db_tests.rs             # Database operation tests
├── pull_wallets                # Kill switch: wallet tracking (create to enable)
├── pull_arkm                   # Kill switch: Arkham enrichment (create to enable)
├── pull_amount                 # Kill switch: amount collection (create to enable)
└── nodejs/                     # Reference Node.js implementation
```

## Environment Variables

| Variable | Required | Description |
|----------|----------|-------------|
| `TURSO_DATABASE_URL` | Yes | Turso database URL (e.g., `turso://your-db.turso.io`) |
| `TURSO_AUTH_TOKEN` | No | Turso authentication token |
| `TRON_TOKEN` | Yes | TronScan API key |
| `ARKM_API_KEY` | No | Arkham Intelligence API key |

## Database Schema

Schema is defined via Toasty ORM models in `src/schema.rs` and created automatically on startup via `ensure_schema()`.

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
| `updated_at` | INTEGER | Last update timestamp (epoch seconds) |
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
| `updated_at` | INTEGER | Last update timestamp (epoch seconds) |

### `mv_user_wallets_groups_export`

Legacy table for checking previously processed wallets.

| Column | Type | Description |
|--------|------|-------------|
| `cc_addresses` | TEXT | Wallet address |

## External APIs

### TronScan API

| Endpoint | Rate Limit | Purpose |
|----------|------------|---------|
| `GET /api/transfer/trc20` | 300ms min | Fetch USDT transfer history (paginated, 50/page) |
| `GET /api/accountv2` | 300ms min | Fetch account metadata |
| `GET /api/account/wallet` | 300ms min | Fetch wallet USD value |
| `GET /api/deep/account/holderToken/basicInfo/trc20/transfer` | 300ms min | Fetch USDT transfer summary |

### Arkham Intelligence API

| Endpoint | Rate Limit | Purpose |
|----------|------------|---------|
| `POST /intelligence/address_enriched/batch` | 1100ms min | Batch wallet enrichment |

## Seed Wallets

When the database is empty, the pipeline starts with these 5 hardcoded TRON wallets:

- `TX..........`

## Development

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

### Key Implementation Details

1. **Schema Management**: Uses `ensure_schema()` which checks if tables exist before creating them (avoids "table already exists" errors on restart).

2. **Toasty ORM**: Uses `toasty::sql::query()` for complex queries (COUNT, JOINs) and Toasty's query builder for simple CRUD operations.

3. **Column Name Mapping**: Uses `#[column("transferIn")]` attribute to map Rust snake_case field names to database camelCase column names.

4. **Connection Mode**: Uses `toasty-driver-turso` with `serverless` feature for direct HTTP connection to Turso Cloud.

5. **Rate Limiting**: Uses `governor` crate for API rate limiting (300ms for TronScan, 1100ms for Arkham).

### Testing

Tests use `Turso::in_memory()` for isolated in-memory databases:

```bash
cargo test
```

### Deployment

For production deployment, use a process manager like systemd or PM2. See `README.md` for systemd service configuration.

## License

ISC

---

## LLM Development Guide

### Coding Conventions

- Use `anyhow::Result` for error handling, not `unwrap()` or `expect()` in production code
- Use `tracing::info!`, `tracing::error!` for logging (not `println!`)
- Use `#[allow(clippy::too_many_arguments)]` for functions with many parameters
- Keep `Option<T>` fields in database models for nullable columns
- Use `snake_case` for Rust fields, map to `camelCase` DB columns with `#[column("camelCase")]`

### File Purposes

| File | Purpose | When to Edit |
|------|---------|--------------|
| `src/main.rs` | Entry point, pipeline orchestration | When changing startup flow or adding new CLI args |
| `src/schema.rs` | Toasty ORM model definitions | When adding/modifying database tables |
| `src/db.rs` | Database operations | When changing queries or adding new DB functions |
| `src/tracker.rs` | Core tracking logic | When modifying the tracking pipeline |
| `src/tronscan.rs` | TronScan API client | When API endpoints change |
| `src/arkham.rs` | Arkham Intelligence client | When Arkham API changes |
| `src/config.rs` | Configuration | When adding new env vars |
| `src/models.rs` | API response structs | When API response formats change |

### Common Patterns

**Database queries:**
```rust
// Simple CRUD - use Toasty query builder
Account::filter_by_wallet(wallet).first().exec(&mut db).await?

// Complex queries - use raw SQL
toasty::sql::query("SELECT COUNT(*) FROM accounts").exec(&mut db).await?
```

**Error handling:**
```rust
// Always use .context() for meaningful errors
db.save_account(&account).await.context("Failed to save account")?;
```

**Async patterns:**
```rust
// Clone db for each async operation (Toasty requirement)
let mut db = self.db.clone();
some_operation(&mut db).await?;
```

### Things to Avoid

- Don't use `unwrap()` in production code
- Don't use `println!` for logging (use `tracing`)
- Don't hardcode API URLs or keys
- Don't use `push_schema()` on every startup (use `ensure_schema()` instead)
- Don't use Toasty's query builder for complex JOINs (use raw SQL)

### Testing

```bash
cargo test           # Run all tests
cargo test arkham    # Run only arkham tests
cargo test db        # Run only db tests
```

Tests use `Turso::in_memory()` for isolated databases. Each test gets a fresh in-memory database.
