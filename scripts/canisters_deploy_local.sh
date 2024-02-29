#!/usr/bin/env bash
set -euo pipefail

cd ./../estate_dao_nft
dfx deploy estate_dao_nft_backend --network=local

gzip -f -1 ./target/wasm32-unknown-unknown/release/estate_dao_nft_backend.wasm

# Specify the path to your Wasm.gz file
minter_wasm="./target/wasm32-unknown-unknown/release/estate_dao_nft_backend.wasm.gz"

# Use xxd to convert the file content to a hexadecimal string
minter_wasm_char=$(hexdump -ve '1/1 "%.2x"' "$minter_wasm")

# Escape special characters in the hexadecimal string
minter_wasm_char_escaped=$(printf "%s" "$minter_wasm_char" | sed 's/../\\&/g')

cd ../provision_canister

printf "( blob \"%s\")" "$minter_wasm_char_escaped" > minter_wasm.blob

dfx deploy provision_canister_backend
dfx deploy internet_identity

# Specify the path to your Wasm.gz file
asset_wasm="./src/provision_canister_backend/assetstorage.wasm.gz"

# Use xxd to convert the file content to a hexadecimal string
asset_wasm_char=$(hexdump -ve '1/1 "%.2x"' "$asset_wasm")

# Escape special characters in the hexadecimal string
asset_wasm_char_escaped=$(printf "%s" "$asset_wasm_char" | sed 's/../\\&/g')

printf "( blob \"%s\")" "$asset_wasm_char_escaped" > asset_wasm.blob


dfx canister call provision_canister_backend init_minter_wasm --argument-file minter_wasm.blob
dfx canister call provision_canister_backend init_asset_wasm --argument-file asset_wasm.blob

rm mint_wasm.blob
rm asset_wasm.blob

can_id=$(jq -r '.provision_canister_backend.local' ./.dfx/local/canister_ids.json)
dfx canister deposit-cycles 8000000000000 $can_id

dfx canister call provision_canister_backend update_key "admin"
