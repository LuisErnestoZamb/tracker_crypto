# Tracker Crypto — TRON USDT Wallet Tracker

A batch processing pipeline that tracks USDT (TRC-20) transactions on the TRON network. It enriches wallet data using TronScan and Arkham Intelligence APIs, filters out exchange wallets, and stores everything in a Turso/libSQL database.

## Architecture

```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│  TronScan    │────▶│  index.js    │────▶│   Turso /    │
│  API         │     │  (pipeline)  │     │   SQLite DB  │
└──────────────┘     └──────┬───────┘     └──────────────┘
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

- Node.js >= 18
- [Turso CLI](https://docs.turso.tech/cli) (for local development)
- API keys:
  - **TronScan** — Get from [TronScan API](https://apilist.tronscanapi.com/)
  - **Arkham Intelligence** — Get from [Arkham](https://platform.arkhamintelligence.com/)

## Setup

```bash
# Clone the repository
git clone <repo-url>
cd tracker_crypto

# Install dependencies
cd nodejs
npm install

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

## Usage

### Running the Pipeline

```bash
# Start with PM2 (recommended)
npx pm2 start index.js

# Stop
npx pm2 stop index.js

# View logs
pm2 logs
```

### Available Scripts

| Script | Description |
|--------|-------------|
| `index.js` | Main tracking pipeline (runs both phases) |
| `replica.js` | Parallel worker with offset 1000 for faster processing |
| `amount.js` | Collects total USD amounts for wallets |
| `arkm.js` | Runs Arkham enrichment only |

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

Rate limiting is handled by [Bottleneck](https://www.npmjs.com/package/bottleneck).

## Database Schema

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
| `is_tracked_arkm` | INTEGER/NULL | 1 if Arkham enrichment completed |
| `is_receiver` | INTEGER (0/1) | 1 if wallet received funds |
| `transferIn` | INTEGER | Inbound transfer count |
| `transferOut` | INTEGER | Outbound transfer count |
| `transactionsTron` | INTEGER | Total TRON transaction count |
| `balanceTron` | INTEGER | TRON balance |
| `deep` | INTEGER | Depth level |
| `payload_tronscan` | TEXT/JSON | Full TronScan API response |
| `arkham_label` | TEXT | Label from Arkham |
| `populated_tags` | TEXT/JSON | Arkham tags array |
| `is_exchange_arkm` | INTEGER (0/1) | Exchange flag from Arkham |
| `is_contract_arkm` | INTEGER (0/1) | Contract flag from Arkham |
| `payload_arkm` | TEXT/JSON | Full Arkham API response |
| `is_amount_collected` | INTEGER (0/1) | Whether USD amount was fetched |
| `total_usd_amount` | REAL | Total USD value of wallet |

### `transactions`

Stores USDT transfer records.

| Column | Type | Description |
|--------|------|-------------|
| `wallet_from` | TEXT | Sender address |
| `wallet_to` | TEXT | Receiver address |
| `wallet_from_id` | INTEGER (FK) | References `accounts.id` |
| `wallet_to_id` | INTEGER (FK) | References `accounts.id` |
| `amount` | REAL | USDT transfer amount |
| `status` | INTEGER | Transaction status |
| `approval_amount` | REAL | Approval amount |
| `block_timestamp` | INTEGER | Block timestamp |
| `block` | INTEGER | Block number |
| `hash_tx` | TEXT (UNIQUE) | Transaction hash |
| `confirmed` | INTEGER | Confirmation status |
| `contract_type` | TEXT | Contract type |
| `revert` | INTEGER | Revert flag |
| `contract_ret` | TEXT | Contract return status |
| `event_type` | TEXT | Event type |
| `issue_address` | TEXT | Token issuer address |
| `decimals` | INTEGER | Token decimals |
| `direction` | INTEGER | Transfer direction |

## Project Structure

```
tracker_crypto/
├── README.md
└── nodejs/
    ├── index.js            # Main pipeline
    ├── track.js            # Core tracking logic
    ├── register.js         # Database operations (SQL)
    ├── connector.js        # Turso/libSQL connection
    ├── replica.js          # Parallel worker (offset 1000)
    ├── amount.js           # USD amount collection
    ├── arkm.js             # Arkham enrichment only
    ├── intel.js            # Test/utility for Arkham API
    ├── compose.yml         # Docker Compose (PostgreSQL + pgAdmin)
    ├── AWS.md              # AWS deployment instructions
    ├── macro.sh            # Curl examples for Arkham API
    ├── pull_wallets        # Kill switch: wallet tracking
    ├── pull_arkm           # Kill switch: Arkham enrichment
    ├── pull_amount         # Kill switch: amount collection
    └── package.json
```

## Deployment

See [AWS.md](nodejs/AWS.md) for deployment instructions.

## License

ISC
