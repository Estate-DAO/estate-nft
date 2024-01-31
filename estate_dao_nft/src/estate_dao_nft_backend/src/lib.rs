mod state;

use candid::IDLArgs;
use candid::{types::number::Nat, CandidType, Deserialize, Principal};
use state::{Account, AdditionalDetails, AdditionalMetadata, CollectionMetadata, FinancialDetails, MarketDetails, Metadata, NFTMetadata, PropDetails, PropertyData, Status};
use serde::Serialize;

use ic_cdk::{query, update, init, caller};
use std::clone;
use std::{cell::RefCell};
use std::collections::{BTreeMap, HashMap};

type NFTList = BTreeMap<String, NFTMetadata>;
// todo tbd CollectionList
type CollectionList = BTreeMap<String, CollectionMetadata>;
type TokenOwnerMap = BTreeMap<String, Principal>;
type UserTokensList = BTreeMap<Principal, Vec<String>>;

pub type Subaccount = [u8; 32];
pub const DEFAULT_SUBACCOUNT: &Subaccount = &[0; 32];

thread_local! {
    static NFT_STORE: RefCell<NFTList> = RefCell::default();
    static COLLECTION_DATA: RefCell<CollectionMetadata> = RefCell::default();
    static COUNTER: RefCell<u16> = RefCell::new (0u16);
    static COLLECTION_ID: RefCell<String> = RefCell::new(String::default());
    static TOKEN_OWNER: RefCell<TokenOwnerMap> = RefCell::default();
    static TOKEN_LIST: RefCell<UserTokensList> = RefCell::default();

}

//collection specific data
// #[update(guard = "allow_only_authorized_principal")] 
#[update] 
fn init_collection(
    name: String, 
    desc: String, 
    owner: Principal,
) -> Result<String, String> {

    COLLECTION_DATA.with(|coll_data| {

        let total_minted = COUNTER.with(|counter| *counter.borrow());

        //using static data for now
        *coll_data.borrow_mut() = 
            CollectionMetadata{
                collection_id:"1".to_string(),
                name,
                desc,
                total_supply:total_minted,
                supply_cap:10000u16,
                property_images:Vec::new(),
                prop_data: None,
                prop_details:None, 
                additional_metadata: Some(AdditionalMetadata{additional_details:None, financial_details:None, documents:Vec::new(), market_details:None}),
                status: Status::Upcoming,
                owner: Principal::to_text(&owner)
        };
    });

    COLLECTION_ID.with(|coll_id| {
        *coll_id.borrow_mut() = "1".to_string();
    });

    return Ok("collection created succesfully".to_string());

}

// //collection specific data
// // #[update(guard = "allow_only_authorized_principal")] 
#[update] 
fn update_prop_det( 
    prop_det: PropDetails
) -> Result<String, String> {

    COLLECTION_DATA.with(|coll_data| {

        let mut col_data= coll_data.borrow_mut().to_owned();
        let user = Principal::from_text(col_data.owner.clone()).unwrap();
        if caller() != Principal::self_authenticating(user) {
            return Err("unathorized user".to_string());
        }
        else {
            // if col_data.prop_details.is_some() {
            //     return Err("property details already added".to_string());
            // }

            col_data.prop_details = Some(prop_det);

            *coll_data.borrow_mut() = col_data;

            return Ok("property details added succesfully".to_string());
        }
    })
}

// //property data
// // #[update(guard = "allow_only_authorized_principal")] 
#[update] 
fn update_prop_data( 
    prop_data: PropertyData
) -> Result<String, String> {

    COLLECTION_DATA.with(|coll_data| {

        let mut col_data= coll_data.borrow_mut().to_owned();
        let user = Principal::from_text(col_data.owner.clone()).unwrap();
        if caller() != Principal::self_authenticating(user) {
            return Err("unathorized user".to_string());
        }
        else {
            // if col_data.prop_details.is_some() {
            //     return Err("property details already added".to_string());
            // }

            col_data.prop_data = Some(prop_data);

            *coll_data.borrow_mut() = col_data;

            //remove
            // let collection_data_test = COLLECTION_DATA.with(|coll_data| { 
            //     coll_data.borrow().to_owned() }); 

            return Ok("property data added succesfully".to_string());
        }
    })
}


// //collection specific data
// // #[update(guard = "allow_only_authorized_principal")] 
#[update] 
fn update_market_details( 
    market_det: MarketDetails
) -> Result<String, String> {

    COLLECTION_DATA.with(|coll_data| {

        let mut col_data= coll_data.borrow_mut().to_owned();
        
        let user = Principal::from_text(col_data.owner.clone()).unwrap();
        if caller() != Principal::self_authenticating(user) {
            return Err("unathorized user".to_string());
        }
        else {
            // if col_data.additional_metadata.clone().unwrap().market_details.is_some() {
            //     return Err("property market details already added".to_string());
            // }
            let mut add_meta = col_data.additional_metadata.unwrap();

            add_meta.market_details = Some(market_det);
            col_data.additional_metadata = Some(add_meta);

            *coll_data.borrow_mut() = col_data;

            return Ok("market details added succesfully".to_string());
        }
    })
}

// //collection specific data
// // #[update(guard = "allow_only_authorized_principal")] 
#[update] 
fn update_financial_details( 
    financial_det: FinancialDetails
) -> Result<String, String> {

    COLLECTION_DATA.with(|coll_data| {
        
        let mut col_data= coll_data.borrow_mut().to_owned();
        let user = Principal::from_text(col_data.owner.clone()).unwrap();
        if caller() != Principal::self_authenticating(user) {
            return Err("unathorized user".to_string());
        }
        else {
        // if col_data.additional_metadata.clone().unwrap().financial_details.is_some() {
        //     return Err("property financial details already added".to_string());
        // }
            let mut add_meta = col_data.additional_metadata.unwrap();

            add_meta.financial_details = Some(financial_det);
            col_data.additional_metadata = Some(add_meta);

            *coll_data.borrow_mut() = col_data;

            return Ok("financial details added succesfully".to_string());
        }
    })
}

// //collection specific data
// // #[update(guard = "allow_only_authorized_principal")] 
#[update] 
fn update_additional_details( 
    add_det: AdditionalDetails
) -> Result<String, String> {

    COLLECTION_DATA.with(|coll_data| {

        let mut col_data= coll_data.borrow_mut().to_owned();

        // if col_data.additional_metadata.clone().unwrap().additional_details.is_some() {
        //     return Err("property additional details already added".to_string());
        // }

        let user = Principal::from_text(col_data.owner.clone()).unwrap();
        if caller() != Principal::self_authenticating(user) {
            return Err("unathorized user".to_string());
        }
        else {
            let mut add_meta = col_data.additional_metadata.unwrap();

            add_meta.additional_details = Some(add_det);
            col_data.additional_metadata = Some(add_meta);

            *coll_data.borrow_mut() = col_data;

            return Ok("additional details succesfully".to_string());
        }
    })
}

//collection specific data
// #[update(guard = "allow_only_authorized_principal")] 
#[update] 
fn init_additional_metadata( 
    add_metadata: AdditionalMetadata,
    prop_details: PropDetails
) -> Result<String, String> {

    COLLECTION_DATA.with(|coll_data| {

        let mut col_data= coll_data.borrow_mut().to_owned();

        if col_data.additional_metadata.is_some() {
            return Err("additional details already added".to_string());
        }
        if col_data.prop_details.is_some() {
            return Err("property details already added".to_string());
        }
        col_data.additional_metadata = Some(add_metadata);

        col_data.prop_details = Some(prop_details);

        *coll_data.borrow_mut() = col_data;

        //remove
        let _collection_data_test = COLLECTION_DATA.with(|coll_data| { 
            coll_data.borrow().to_owned() }); 

        return Ok("collection created succesfully".to_string());

    })
}


// //collection specific data
// // #[update(guard = "allow_only_authorized_principal")] 
#[update] 
fn update_doc_details(
    doc_details: Vec<HashMap<String, String>>
) -> Result<String, String> {

    COLLECTION_DATA.with(|coll_data| {

        let mut col_data= coll_data.borrow_mut().to_owned();

        let user = Principal::from_text(col_data.owner.clone()).unwrap();
        if caller() != Principal::self_authenticating(user) {
            return Err("unathorized user".to_string());
        }
        else {
            
            let mut add_meta = col_data.additional_metadata.unwrap();

            add_meta.documents = doc_details;
            col_data.additional_metadata = Some(add_meta);

            *coll_data.borrow_mut() = col_data;

            return Ok("Documents added succesfully".to_string());
        }
    })
}


// Metadata items TBD
// for now using NFTMetadata + Collection Metadata 
#[query] 
fn get_collection_metadata() -> Result<CollectionMetadata, String> {

    let collection_data: CollectionMetadata = COLLECTION_DATA.with(|coll_data| { 
        coll_data.borrow().to_owned() });
            
    return Ok(collection_data);
}

// for now using NFTMetadata + Collection Metadata 
#[query] 
fn get_collection_status() -> Result<Status, String> {

    let collection_data: CollectionMetadata = COLLECTION_DATA.with(|coll_data| { 
        coll_data.borrow().to_owned() });
            
    return Ok(collection_data.status);
}


#[query] 
fn get_prop_data() -> Result<PropertyData, String> {

    let collection_data: CollectionMetadata = COLLECTION_DATA.with(|coll_data| { 
        coll_data.borrow().to_owned() });
            
    return Ok(collection_data.prop_data.unwrap());
}


// Metadata items TBD
// for now using NFTMetadata + Collection Metadata 
#[query] 
fn get_market_details() -> Result<MarketDetails, String> {

    let collection_data: CollectionMetadata = COLLECTION_DATA.with(|coll_data| { 
        coll_data.borrow().to_owned() });
            
    return Ok(collection_data.additional_metadata.unwrap().market_details.unwrap());
}

// TODO update total_minted in collectionMetadata
// #[update(guard = "allow_only_authorized_principal")] 
#[update] 
fn mint(symbol: String, uri: String, owner: Principal) -> Result<String, String> {

    let counter = COUNTER.with(|counter| {
        *counter.borrow_mut() += 1;
        *counter.borrow()
    });

    let collection_id = COLLECTION_ID.with(|coll_id| (
            *coll_id.borrow()).clone());

    COLLECTION_DATA.with(|coll_data| {
        let mut col_data= coll_data.borrow_mut().to_owned();
        col_data.total_supply = counter;
    });

    TOKEN_OWNER.with(|token_owner_map| {
        let mut owner_list =  token_owner_map.borrow_mut();
        owner_list.insert(counter.clone().to_string(), owner.clone());
    });    
    

    NFT_STORE.with(|nft_list| {
        nft_list.borrow_mut().insert(
            counter.clone().to_string(),
            NFTMetadata{
                collection_id,
                nft_symbol: symbol,
                nft_token_id: counter.to_string(),
                nft_uri: uri
            }
        )
    });

    return Ok("NFT minted succesfully".to_string());
}

// Metadata items TBD
// for now using NFTMetadata + Collection Metadata 
#[query] 
fn get_metadata(token_id : String) -> Result<Metadata, String> {

    NFT_STORE.with(|nft_list| {
        let nft_lists = nft_list.borrow();
        let nft_data = nft_lists.get(&token_id);

        if nft_data.is_none() {
            return Err(String::from("NFT not found"));
        }
        let nft_data = nft_data.unwrap().to_owned();

        let collection_data: CollectionMetadata = COLLECTION_DATA.with(|coll_data| { 
            coll_data.borrow().to_owned() });

            let metadata = Metadata{
                collection_id: collection_data.collection_id,
                nft_symbol: nft_data.nft_symbol,
                nft_token_id: nft_data.nft_token_id,
                nft_uri: nft_data.nft_uri,
                collection_name: collection_data.name,
                desc: collection_data.desc,
                total_supply: collection_data.total_supply,
                supply_cap: collection_data.supply_cap
            };

            return Ok(metadata);
    }) 
}




// #[update(guard = "allow_only_authorized_principal")] 
// fn add_collection_image(asset_canister_id: String, image: String) -> Result<CollectionMetadata, String> {
//     let image_uri = "https://".to_owned() + &asset_canister_id + ".icp0.io/" + &image;
        
//     // let mut collection_data: CollectionMetadata = COLLECTION_DATA.with(|coll_data| { 
//     //     coll_data.borrow_mut().to_owned() });

//     COLLECTION_DATA.with(|coll_data| {
        
//         let mut col_data= coll_data.borrow_mut().to_owned();
//         col_data.property_images.push(image_uri);

//         *coll_data.borrow_mut() = col_data;

//         //remove
//         let collection_data_test = COLLECTION_DATA.with(|coll_data| { 
//             coll_data.borrow().to_owned() }); 
            
//         return Ok(collection_data_test);

//     })
// }

#[query] 
fn collection_image() -> Vec<String>{

    let collection_data = COLLECTION_DATA.with(|coll_data| { 
        coll_data.borrow().to_owned() });
      
    collection_data.property_images
}

ic_cdk::export_candid!();

/////////////////////
fn allow_only_authorized_principal() -> Result<(), String> {
    let authorized_principal_id = Principal::from_text("2ghx4-leaaa-aaaaa-qacru-cai").unwrap();
    if caller() != authorized_principal_id {
        Err(String::from("Access denied"))
    } else {
        Ok(())
    }
}