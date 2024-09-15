# Solana Copy Trade Detect

This crate provides functionality to detect if a given Solana wallet is copy trading.

```bash
Detects if a given Solana wallet is copy trading.

Usage: solana-copy-trade-detect [OPTIONS] --cielo-api-key <CIELO_API_KEY> --shyft-api-key <SHYFT_API_KEY> --wallet <WALLET>

Options:
  -c, --cielo-api-key <CIELO_API_KEY>  Cielo API key [env: CIELO_API_KEY=]
  -s, --shyft-api-key <SHYFT_API_KEY>  Shyft API key [env: SHYFT_API_KEY=]
  -r, --rpc-url <RPC_URL>              Solana RPC URL. The Shyft RPC endpoint is used by default if not provided [env: RPC_URL=]
  -w, --wallet <WALLET>                The wallet to scan
      --swap-num <SWAP_NUM>            Number of recent fresh swaps from the wallet to consider (max 100) [default: 15]
      --scan-tx-count <SCAN_TX_COUNT>  Number of transactions to scan for each swap to detect repeated wallets (max 100) [default: 50]
  -d, --delay-ms <DELAY_MS>            Delay between Shyft API requests in milliseconds [default: 500]
  -o, --output-file <OUTPUT_FILE>      Output file to write detected wallets. Default is wallet_address.txt
  -h, --help                           Print help
  -V, --version                        Print version
  ```

## Description

- Fresh swaps are the initial swap transactions of different token pairs from the specified wallet. These are fetched from the Cielo API feed
- Scanned transactions refer to the token's transaction history before the fresh swap. These transactions are retrieved using the Shyft API.
- If a wallet address appears repeatedly in the transaction history of various fresh swaps, it suggests that the given wallet might be copying the transactions of that address.
