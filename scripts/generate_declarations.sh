#!/usr/bin/env bash
set -euo pipefail

cd ./../estate_dao_nft
dfx generate estate_dao_nft_backend

cd ./../provision_canister
dfx generate provision_canister_backend