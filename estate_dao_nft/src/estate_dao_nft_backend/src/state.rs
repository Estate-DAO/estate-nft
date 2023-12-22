use serde::Serialize;
use candid::{types::number::Nat, CandidType, Deserialize, Principal};
// use icrc_ledger_types::icrc1::account::Account;

pub type Subaccount = [u8; 32];
pub const DEFAULT_SUBACCOUNT: &Subaccount = &[0; 32];


#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct Account{ 
    pub owner: Principal,
    pub subaccount: Option<Subaccount>
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct RoyaltyData{
    pub royalty_share: u16,
    pub royalty_account: Account,
}

//Collection level metadata
//Royalty structure is not fixed yet in ICRC7
#[derive(Clone, Debug, Default, CandidType, Deserialize, Serialize)]
pub struct CollectionMetadata {
    pub collection_id: String,
    pub name: String,
    pub desc: String,
    pub logo: String, //collection logo 
    // pub royalty: RoyaltyData,
    pub royalty_percent: u16,
    pub total_supply: u16,
    pub supply_cap: u16,
    // pub property_images: Vec<String>,
}

// NFT specific data
#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct NFTMetadata {
    pub collection_id: String,
    pub nft_symbol: String,
    pub nft_token_id: String,
    pub nft_uri: String //image
}

// NFTMetadata + CollectionMetadata
#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct Metadata {
    pub collection_id: String,
    pub nft_symbol: String,
    pub nft_token_id: String,
    pub nft_uri: String, //image   
    pub collection_name: String,
    pub desc: String,
    pub logo: String, //collection logo 
    // pub royalty: RoyaltyData,
    pub royalty_percent: u16,
    pub total_supply: u16,
    pub supply_cap: u16,
}