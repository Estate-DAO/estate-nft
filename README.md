# Estate-DAO Backend Canisters

# Verifying builds

To get the hash for canisters:

- Get the canister IDs from [`canister_ids.json`](https://github.com/Estate-DAO/estate-nft/blob/main/provision_canister/canister_ids.json).
- Get hash using the DFX SDK by running: `dfx canister info <canister-id> --network=ic`.

- The output of the above command should contain `Module hash` followed up with the hash value. Example output:

  ```
  $ > dfx canister info vyatz-hqaaa-aaaam-qauea-cai --network=ic

  Controllers: v3mpp-bismc-wjug7-6t6jc-iqu2b-64xh3-rpwld-sw5e2-fsecm-6lfss-aqe
  Module hash: 0x1682ca6286cbbeef23abfe7c3d35b8e1d1dec884b40696637e40bc6fe1ac1e3a
  ```

To get the hash for canister deployment:

- Go to [Github actions deployment runs](https://github.com/Estate-DAO/estate-nft/actions/workflows/deploy.yml)
- Open the latest succesful run. ([Click to see an example run](https://github.com/Estate-DAO/estate-nft/actions/runs/8389802581))
- Go to any of the `Deploy all canisters` jobs. ([Click to see an example job](https://github.com/Estate-DAO/estate-nft/actions/runs/8389802581/job/22976747707))
- Open the `Deploy canister` step. You should find the `Module hash` in this step. This value should match the value you got locally. ([Click to see an example step]([https://github.com/go-bazzinga/hot-or-not-backend-canister/actions/runs/4900015913/jobs/8750374252#step:8:16](https://github.com/Estate-DAO/estate-nft/actions/runs/8389802581/job/22976747707)https://github.com/Estate-DAO/estate-nft/actions/runs/8389802581/job/22976747707))

---
