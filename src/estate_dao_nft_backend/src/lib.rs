mod state;

use candid::{types::number::Nat, CandidType, Deserialize, Principal};
use state::{NFTMetadata, CollectionMetadata, Account};
use serde::Serialize;

use ic_cdk::{query, update, init};
use std::{cell::RefCell, fs::Metadata};
use std::collections::BTreeMap;

type NFTList = BTreeMap<String, NFTMetadata>;

pub type Subaccount = [u8; 32];
pub const DEFAULT_SUBACCOUNT: &Subaccount = &[0; 32];

thread_local! {
    static NFT_STORE: RefCell<NFTList> = RefCell::default();
    static COLLECTION_DATA: RefCell<CollectionMetadata> = RefCell::default();
    static COUNTER: RefCell<u16> = RefCell::new (0u16);
    static COLLECTION_ID: RefCell<String> = RefCell::new(String::default());
}

//collection specific data
#[init]
fn insert_collection_data() {
    
    COLLECTION_DATA.with(|coll_data| {

        let total_minted = COUNTER.with(|counter| *counter.borrow());

        // CollectionMetadata{ 
        //     collection_id: collection_id.clone(),
        //     name: coll_metadata.name,
        //     desc: coll_metadata.desc,
        //     logo: coll_metadata.logo,
        //     royalty_percent: coll_metadata.royalty_percent,
        //     total_supply: total_minted,
        //     supply_cap: coll_metadata.supply_cap,
        // }

        //using static data for now
        *coll_data.borrow_mut() = CollectionMetadata{ 
            collection_id: "1".to_string(),
            name: "collection dummy".to_string(),
            desc: "description".to_string(),
            logo: "uri".to_string(),
            royalty_percent: 300u16,
            total_supply: total_minted,
            supply_cap: 10000u16
        }
    });

    COLLECTION_ID.with(|coll_id| {
        *coll_id.borrow_mut() = "1".to_string();
    });


}


// TODO add check for controller account
// #[update]
// fn update_collection_data() -> Result<String, String> {}


#[update]
fn mint(symbol: String, uri: String) -> Result<String, String> {

    let counter = COUNTER.with(|counter| {

        *counter.borrow_mut() += 1;
        
        let id = *counter.borrow();

        let collection_id = COLLECTION_ID.with(|coll_id| (*coll_id.borrow()).clone());

        NFT_STORE.with(|nft_list| {
            nft_list.borrow_mut().insert(
                id.clone().to_string(),
                NFTMetadata{
                    collection_id,
                    nft_symbol: symbol,
                    nft_token_id: id.to_string(),
                    nft_uri: uri
                }
            )
        });


    });

    return Ok("NFT minted succesfully".to_string());
}

// Metadata items TBD
// for now using NFTMetadata + Collection Metadata 
#[query] 
fn get_(token_id : String) -> Result<(NFTMetadata, CollectionMetadata), String> {

    NFT_STORE.with(|nft_list| {
        let nft_lists = nft_list.borrow();
        let nft_data = nft_lists.get(&token_id);

        if nft_data.is_none() {
            return Err(String::from("NFT not found"));
        }

        let collection_data: CollectionMetadata = COLLECTION_DATA.with(|coll_data| { 
                coll_data.borrow().to_owned() });
        
        return Ok((nft_data.unwrap().clone(), collection_data));

    }) 
}

ic_cdk::export_candid!();
