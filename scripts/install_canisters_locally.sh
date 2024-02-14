#!/usr/bin/env bash
set -euo pipefail

cd ./estate_dao_nft
dfx deploy estate_dao_nft_backend --network=local

gzip -f -1 ./target/wasm32-unknown-unknown/release/estate_dao_nft_backend.wasm

cd ../provision_canister

dfx deploy provision_canister_backend
dfx deploy internet_identity