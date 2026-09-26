# Tracker Crypto — TRON USDT Wallet Tracker

A batch processing pipeline that tracks USDT (TRC-20) transactions on the TRON network. It enriches wallet data using TronScan and Arkham Intelligence APIs, filters out exchange wallets, and stores everything in a Turso database.

## Quick Start

```bash
# Build
cargo build --release

# Configure
cat << EOF > .env
TURSO_DATABASE_URL=turso://your-db.turso.io
TURSO_AUTH_TOKEN=your_token
TRON_TOKEN=your_tronscan_api_key
ARKM_API_KEY=your_arkham_api_key
EOF

# Create kill switch files (to enable loops)
touch pull_wallets pull_arkm pull_amount

# Run
cargo run --release
```

## How It Works

1. **First run**: Seeds 5 hardcoded TRON wallets and starts tracking
2. **Subsequent runs**: Processes wallets from the database
3. **Enrichment**: Identifies exchanges and labels via Arkham Intelligence
4. **Collection**: Fetches USD value for each wallet

## Controlling the Pipeline

The pipeline uses file-based kill switches. Create or delete files to control which loops run:

| File | Controls |
|------|----------|
| `pull_wallets` | Wallet tracking loop |
| `pull_arkm` | Arkham enrichment loop |
| `pull_amount` | USD amount collection loop |

```bash
# Enable a loop
touch pull_wallets

# Disable a loop
rm pull_wallets
```

## Environment Variables

| Variable | Required | Description |
|----------|----------|-------------|
| `TURSO_DATABASE_URL` | Yes | Turso database URL |
| `TURSO_AUTH_TOKEN` | No | Turso authentication token |
| `TRON_TOKEN` | Yes | TronScan API key |
| `ARKM_API_KEY` | No | Arkham Intelligence API key |

## Project Structure

```
tracker_crypto/
├── src/
│   ├── main.rs           # Entry point
│   ├── schema.rs         # Database models (Toasty ORM)
│   ├── db.rs             # Database operations
│   ├── tracker.rs        # Core tracking logic
│   ├── tronscan.rs       # TronScan API client
│   ├── arkham.rs         # Arkham Intelligence client
│   ├── config.rs         # Configuration
│   └── models.rs         # API response structs
├── pull_wallets          # Kill switch: wallet tracking
├── pull_arkm             # Kill switch: Arkham enrichment
└── pull_amount           # Kill switch: amount collection
```

## Development

```bash
cargo build          # Build in debug mode
cargo test           # Run tests
cargo clippy         # Run linter
cargo fmt            # Format code
```

## License

ISC
