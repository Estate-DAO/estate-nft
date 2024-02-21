mod state;

use candid::{types::number::Nat, CandidType, Deserialize, Principal};
use ic_cdk::api::time;
use state::{AdditionalMetadata, CollectionMetadata, FinancialDetails, FormMetadata, MarketDetails, Metadata, NFTMetadata, PropertyDetails, SaleData, SaleStatus, Status};
use serde::Serialize;

use ic_cdk::{api::call::call, query, update, init, caller};
use std::{clone, string};
use std::{cell::RefCell};
use std::collections::{BTreeMap, HashMap};

use ic_ledger_types::{
    AccountBalanceArgs, AccountIdentifier, Memo, Subaccount, Tokens, DEFAULT_SUBACCOUNT, MAINNET_LEDGER_CANISTER_ID
};
use icrc_ledger_types::{
    icrc1::account::Account, 
    icrc2::{
        allowance::{Allowance, AllowanceArgs},
        transfer_from::{TransferFromArgs, TransferFromError}
    }
};

type NFTList = BTreeMap<String, NFTMetadata>;
// todo tbd CollectionList
type TokenOwnerMap = BTreeMap<String, Account>;
type UserTokensList = BTreeMap<Account, Vec<String>>;
type SaleList = BTreeMap<String, SaleData>;

#[derive(Clone, Debug, CandidType, Default, Deserialize, Serialize)]
pub struct CanisterData { 
    pub collection_data: CollectionMetadata,
    pub nft_store: NFTList,
    pub token_owner: TokenOwnerMap,
    pub sales_data: SaleList,
    pub total_invested: u64
}

thread_local! {
    static CANISTER_DATA: RefCell<CanisterData> = RefCell::default()
    // static NFT_STORE: RefCell<NFTList> = RefCell::default();
    // // static COLLECTION_DATA: RefCell<CollectionMetadata> = RefCell::default();
    // static COUNTER: RefCell<u16> = RefCell::new (0u16);
    // static TOKEN_OWNER: RefCell<TokenOwnerMap> = RefCell::default();
    // static TOKEN_LIST: RefCell<UserTokensList> = RefCell::default();
    // static SALE_DATA: RefCell<SaleList> = RefCell::default();
    // static TOTAL_INVESTED: RefCell<u64> = RefCell::new (0u64);
}


//collection specific data
// #[update(guard = "allow_only_authorized_principal")] 
#[update]
fn init_collection(
    form_data: FormMetadata
) -> Result<String, String> {

    CANISTER_DATA.with(|canister_data| {

        // let total_minted = COUNTER.with(|counter| *counter.borrow());
        let mut canister_data_ref = canister_data.borrow_mut();

        if canister_data_ref.collection_data.is_initialised == true {
            // return Err("collection owner not initialized".to_string());
            Err("collection already initialised".to_string())
        }

        else {
            let mut add_metadata = form_data.additional_metadata;
            if add_metadata.is_none() {
                add_metadata = Some(AdditionalMetadata{
                    property_details: None,
                    financial_details: None,
                    documents: Vec::new(),
                    market_details: None                        
                });
            }
             
            canister_data_ref.collection_data = 
                CollectionMetadata{
                    name: form_data.name,
                    desc: form_data.desc,
                    total_supply: 0u16,
                    supply_cap: form_data.supply_cap,
                    property_images: form_data.property_images,
                    additional_metadata: add_metadata,
                    status: Status::Draft,
                    owner: form_data.owner,
                    is_initialised: true
            };

            Ok("collection created succesfully".to_string())
        }
    })
}

#[update] 
fn update_basic_details( 
    name: Option<String>,
    desc: Option<String>,
    stat: Option<Status>
) -> Result<String, String> {

    CANISTER_DATA.with(|canister_data| {
        
        let mut canister_data_ref= canister_data.borrow_mut().to_owned();
        let mut col_data = canister_data_ref.collection_data;

        let user_res = Principal::from_text(col_data.owner.clone());
        let user: Principal;
        match user_res{
            Ok(id) => {user=id;},
            Err(e) => return Err("collection owner not initialized".to_string())
        };
        if caller() != user {
            Err("unathorized user".to_string())
        }
        else {

            match name{
                Some(new_name) => {
                    col_data.name = new_name;
                }
                _ => {}
            }            
            match desc{
                Some(new_description) => {
                    col_data.desc = new_description;
                }
                _ => {}
            }
            match stat{
                Some(new_status) => {
                    col_data.status = new_status;
                }
                _ => {}
            }

            canister_data_ref.collection_data = col_data;
            *canister_data.borrow_mut() = canister_data_ref;

            Ok("basic details updated succesfully".to_string())
        }
    })
}


// //collection specific data
// // #[update(guard = "allow_only_authorized_principal")] 
#[update] 
fn update_market_details( 
    market_det: MarketDetails
) -> Result<String, String> {

    CANISTER_DATA.with(|canister_data| {
        
        let mut canister_data_ref= canister_data.borrow_mut().to_owned();
        let mut col_data = canister_data_ref.collection_data;

        let user_res = Principal::from_text(col_data.owner.clone());
        let user: Principal;
        match user_res{
            Ok(id) => {user=id;},
            Err(e) => return Err("collection owner not initialized".to_string())
        };
        if caller() != user {
            Err("unathorized user".to_string())
        }
        else {
            let mut add_meta = col_data.additional_metadata.ok_or("collection not initialized")?;

            add_meta.market_details = Some(market_det);
            col_data.additional_metadata = Some(add_meta);

            canister_data_ref.collection_data = col_data;
            *canister_data.borrow_mut() = canister_data_ref;


            Ok("market details added succesfully".to_string())
        }
    })
}

// //collection specific data
// // #[update(guard = "allow_only_authorized_principal")] 
#[update] 
fn update_financial_details( 
    financial_det: FinancialDetails
) -> Result<String, String> {

    CANISTER_DATA.with(|canister_data| {
        
        let mut canister_data_ref= canister_data.borrow_mut().to_owned();
        let mut col_data = canister_data_ref.collection_data;

        let user_res = Principal::from_text(col_data.owner.clone());
        let user: Principal;
        match user_res{
            Ok(id) => {user=id;},
            Err(_) => return Err("collection owner not initialized".to_string())
        };
        if caller() != user {
            return Err("unathorized user".to_string());
        }
        else {
            let mut add_meta = col_data.additional_metadata.ok_or("collection not initialized")?;

            add_meta.financial_details = Some(financial_det);
            col_data.additional_metadata = Some(add_meta);

            canister_data_ref.collection_data = col_data;
            *canister_data.borrow_mut() = canister_data_ref;


            return Ok("financial details added succesfully".to_string());
        }
    })
}

// //collection specific data
// // #[update(guard = "allow_only_authorized_principal")] 
#[update] 
fn update_property_details( 
    add_det: PropertyDetails
) -> Result<String, String> {


    CANISTER_DATA.with(|canister_data| {
        
        let mut canister_data_ref= canister_data.borrow_mut().to_owned();
        let mut col_data = canister_data_ref.collection_data;

        let user_res = Principal::from_text(col_data.owner.clone());
        let user: Principal;
        match user_res{
            Ok(id) => {user=id;},
            Err(e) => return Err("collection owner not initialized".to_string())
        };
        if caller() != user {
            return Err("unathorized user".to_string());
        }
        let mut add_meta = col_data.additional_metadata.ok_or("collection not initialized")?;

        add_meta.property_details = Some(add_det);
        col_data.additional_metadata = Some(add_meta);

        canister_data_ref.collection_data = col_data;
        *canister_data.borrow_mut() = canister_data_ref;

        Ok("additional details succesfully".to_string())
    })
}

// //collection specific data
// // #[update(guard = "allow_only_authorized_principal")] 
// // #[update] 
// // fn init_additional_metadata( 
// //     add_metadata: AdditionalMetadata,
// //     prop_details: PropDetails
// // ) -> Result<String, String> {

// //     COLLECTION_DATA.with(|coll_data| {

// //         let mut col_data= coll_data.borrow_mut().to_owned();

// //         if col_data.additional_metadata.is_some() {
// //             return Err("additional details already added".to_string());
// //         }
// //         if col_data.prop_details.is_some() {
// //             return Err("property details already added".to_string());
// //         }
// //         col_data.additional_metadata = Some(add_metadata);

// //         col_data.prop_details = Some(prop_details);

// //         *coll_data.borrow_mut() = col_data;

// //         //remove
// //         let _collection_data_test = COLLECTION_DATA.with(|coll_data| { 
// //             coll_data.borrow().to_owned() }); 

// //         return Ok("collection created succesfully".to_string());

// //     })
// // }


// //collection specific data
// // #[update(guard = "allow_only_authorized_principal")] 
#[update] 
fn update_doc_details(
    doc_details: Vec<HashMap<String, String>> //errorEnum 
) -> Result<String, String> {

    CANISTER_DATA.with(|canister_data| {
        
        let mut canister_data_ref= canister_data.borrow_mut().to_owned();
        let mut col_data = canister_data_ref.collection_data;

        let user_res = Principal::from_text(col_data.owner.clone());
        let user: Principal;
        match user_res{
            Ok(id) => {user=id;},
            Err(e) => return Err("collection owner not initialized".to_string())
        };
        if caller() != user {
            Err("unathorized user".to_string())
        }
        else {
            
            let mut add_meta = col_data.additional_metadata.ok_or("collection not initialized")?;

            add_meta.documents = doc_details;
            col_data.additional_metadata = Some(add_meta);

            canister_data_ref.collection_data = col_data;
            *canister_data.borrow_mut() = canister_data_ref;

            Ok("Documents added succesfully".to_string())
        }
    })
}


// Metadata items TBD
// for now using NFTMetadata + Collection Metadata 
#[query] 
fn get_collection_metadata() -> Result<CollectionMetadata, String> {

    let collection_data = CANISTER_DATA.with(|canister_data: &RefCell<CanisterData>| { 
        canister_data.borrow().collection_data.to_owned() });

    Ok(collection_data)
}

// for now using NFTMetadata + Collection Metadata 
#[query] 
fn get_collection_status() -> Result<Status, String> {

    let collection_data = CANISTER_DATA.with(|canister_data| { 
        canister_data.borrow().collection_data.to_owned() });

    return Ok(collection_data.status);
}

// market details
#[query] 
fn get_market_details() -> Result<FinancialDetails, String> {

    let collection_data = CANISTER_DATA.with(|canister_data| { 
        canister_data.borrow().collection_data.to_owned() });

    Ok(collection_data.additional_metadata.ok_or("collection not initialized")?.financial_details.ok_or("collection not initialized")?)
}

// financial details
#[query] 
fn get_financial_details() -> Result<MarketDetails, String> {

    let collection_data = CANISTER_DATA.with(|canister_data| { 
        canister_data.borrow().collection_data.to_owned() });
        
    Ok(collection_data.additional_metadata.ok_or("collection not initialized")?.market_details.ok_or("collection not initialized")?)
}

// additional details
#[query] 
fn get_property_details() -> Result<PropertyDetails, String> {

    let collection_data = CANISTER_DATA.with(|canister_data| { 
        canister_data.borrow().collection_data.to_owned() });
        
    Ok(collection_data.additional_metadata.ok_or("collection not initialized")?.property_details.ok_or("collection not initialized")?)
}

// TODO update total_minted in collectionMetadata
// #[update(guard = "allow_only_canister")] 
#[update]
fn mint(token_id: String, symbol: String, uri: String, owner: Principal, fail: u16) -> Result<String, String> {

    if fail == 0 {
        return Err("mint set to failed".to_string());
    }
    CANISTER_DATA.with(|canister_data| {
        
        let mut canister_data_ref= canister_data.borrow_mut().to_owned();
        // let mut col_data = canister_data_ref.collection_data;
        let mut token_owner_map = canister_data_ref.token_owner;
        let mut nft_map = canister_data_ref.nft_store;

        let owner_account = Account::from(owner);
        token_owner_map.insert(token_id.clone(), owner_account);
        // });    
        
        // TOKEN_LIST.with(|user_token_list| {
        //     let binding = user_token_list.borrow_mut();
        //     let token_list =  binding.get(&owner);
        //     // token_list.insert(token_id.to_string(), owner.clone())
        //     match token_list {
        //         Some(_v) => {

        //             let mut token_list_map =  user_token_list.clone().borrow_mut().to_owned();
        //             let mut list: Vec<String> = Vec::new();
        //             token_list_map.get(&owner).unwrap().clone_into(&mut list); 
        //             list.push(token_id.to_string());
        //             token_list_map.insert(owner.clone(), list);

        //             // list.push(token_id.to_string());
        //             // token_list_map.insert(owner.clone(), *list);

        //             // let mut list = token_list.unwrap(); 
        //             // token_list.unwrap().push(token_id.to_string());
        //         }
        //         _ => {
        //             let mut token_list =  user_token_list.borrow_mut();
        //             let mut token_vec: Vec<String> = Vec::new();
        //             token_vec.push(token_id.to_string());
        //             token_list.insert(owner.clone(), token_vec);
        //         }
        //     };
        // });

        nft_map.insert(
                token_id.clone(),
                NFTMetadata{
                    nft_symbol: symbol,
                    nft_token_id: token_id,
                    nft_uri: uri
                }
            );
    
        canister_data_ref.token_owner = token_owner_map;
        canister_data_ref.nft_store = nft_map ;
        *canister_data.borrow_mut() = canister_data_ref;

        Ok("NFT minted succesfully".to_string())
    })

}

// Metadata items TBD
// for now using NFTMetadata + Collection Metadata 
#[query] 
fn get_metadata(token_id : String) -> Result<Metadata, String> {

    CANISTER_DATA.with(|canister_data| {

        let canister_data_ref= canister_data.borrow().to_owned();

        let nft_map = canister_data_ref.nft_store;
        let nft_data = nft_map.get(&token_id);
        let nft_data = nft_data.ok_or("NFT not forund")?.to_owned();

        let col_data = canister_data_ref.collection_data;

        let metadata = Metadata{
            nft_symbol: nft_data.nft_symbol,
            nft_token_id: nft_data.nft_token_id,
            nft_uri: nft_data.nft_uri,
            collection_name: col_data.name,
            desc: col_data.desc,
            total_supply: col_data.total_supply,
            supply_cap: col_data.supply_cap
        };
    Ok(metadata)
    }) 
}

#[update]
fn get_owner_of_nft(token_id: String) -> Result<Account, String> {

    CANISTER_DATA.with(|canister_data| {
        
        let canister_data_ref= canister_data.borrow().to_owned();
        let token_owner_map = canister_data_ref.token_owner;

        let token_owner = token_owner_map.get(&token_id);

        Ok(*token_owner.ok_or("invalid token_id")?)
    })
}

#[update]
fn add_collection_image(image: String) -> Result<String, String> {

    CANISTER_DATA.with(|canister_data| {
        
        let mut canister_data_ref= canister_data.borrow_mut().to_owned();
        // let mut col_data = canister_data_ref.collection_data;

        canister_data_ref.collection_data.property_images.push(image);

        *canister_data.borrow_mut() = canister_data_ref;

        Ok("sucess".to_string())
    })
}

#[query] 
fn collection_image() -> Vec<String>{

    let canister_data = CANISTER_DATA.with(|canister_data| { 
        canister_data.borrow().to_owned() });
    let col_data = canister_data.collection_data;
  
    col_data.property_images
}


#[update]
async fn primary_sale(receiver_id: Principal, buyer_id: Principal) -> Result<String, String> {
    let canister_id = ic_cdk::api::id();

    // let canister_data = CANISTER_DATA.with(|canister_data_ref| {

    let collection_data = CANISTER_DATA.with(|canister_data| { 
        canister_data.borrow().collection_data.to_owned() });

    let ledger_canister_id = Principal::from_text("ryjl3-tyaaa-aaaaa-aaaba-cai").unwrap();

    // new token_id
    let token_counter = collection_data.total_supply;

    // todo check allowance
    let allowance_arg = AllowanceArgs{
        account: Account::from(buyer_id),
        spender: Account::from(canister_id),
    };

    let delegated_amount: Allowance;
    let allowance_call_res =  call(ledger_canister_id, "icrc2_allowance", (allowance_arg,), ).await;
    match allowance_call_res{
        Ok(r) => {
            let (res,): (Allowance,) = r;
            delegated_amount = res;
        },
        Err(_e) => todo!()
    }
    let nft_price = collection_data.additional_metadata
        .ok_or("collection additional metadata not initalized")?
        .financial_details.ok_or("unable to fetch prop price".to_string())?
        .investment.ok_or("unable to fetch investment details")?
        .min_investment.ok_or("unable to fetch selling price details")?;

    let receiver_account = collection_data.owner;
    let receiver_id =  Principal::from_text(receiver_account).unwrap();

    // check if allowance > nft price
    if delegated_amount.allowance < nft_price {
        return Err("delegated amount less than NFT price".to_string());
    }

    let mut sale_data = SaleData{
        nft_token_id: ((token_counter + 1).to_string()),
        buyer: Account::from(buyer_id),    
        amount: nft_price,
        status: SaleStatus::Init,
        time: ic_ledger_types::Timestamp { timestamp_nanos: time() }
    };

    let transfer_from_args = TransferFromArgs {
        spender_subaccount: None,
        memo: None,
        amount: Nat::from(nft_price),
        fee: None,
        from: Account::from(buyer_id),
        to: Account::from(receiver_id),
        created_at_time: None,
    };

    let mut transfer_status: u16 = 0;
    let res =  call(ledger_canister_id, "icrc2_transfer_from", (transfer_from_args,), ).await;
    match res{
        Ok(r) => {
            let (res,): (Result<Nat, TransferFromError>,) = r;
            match res{
                Ok(_) => {transfer_status = 1},
                Err(_e) => {transfer_status = 0}
            }
        },
        Err(_e) => {transfer_status = 0}
    }

    if transfer_status == 0{
        return Err("error transfering from canister to seller".to_string());
    }

    let mut counter: u16 = Default::default();    
    CANISTER_DATA.with(|canister_data| {

        let mut canister_data_ref = canister_data.borrow().to_owned();
        canister_data_ref.total_invested = canister_data_ref.total_invested + nft_price;
        canister_data_ref.collection_data.total_supply = canister_data_ref.collection_data.total_supply + 1;
        counter = canister_data_ref.collection_data.total_supply;

        *canister_data.borrow_mut() = canister_data_ref;
    });

    //mint function
    let symbol = collection_data.name +  &counter.to_string();
    let uri = String::from("image url");
    let mint_res = mint(counter.to_string(), symbol, uri, buyer_id, 1);

    match mint_res {
        Ok(r) => {
            sale_data.status = SaleStatus::Complete;

            CANISTER_DATA.with(|canister_data| {

                let mut canister_data_ref= canister_data.borrow_mut();
                canister_data_ref.sales_data.insert(counter.to_string(), sale_data);

            });

            Ok(r)
        },
        Err(e) => {
            sale_data.status = SaleStatus::Incomplete;

            CANISTER_DATA.with(|canister_data| {

                let mut canister_data_ref= canister_data.borrow_mut();
                canister_data_ref.sales_data.insert(counter.to_string(), sale_data);

            });
            Err(e)
        },
    }
}


#[update] 
fn primary_sale_mint(token_id : String) -> Result<String, String> {

    let buyer_id = caller();
    CANISTER_DATA.with(|canister_data| {
        let mut canister_data_ref = canister_data.borrow().to_owned();

        let mut sales_data = canister_data_ref.sales_data;
        
        let mut sale_data = sales_data.get_mut(&token_id).ok_or("unable fetch no sale exist for this token".to_string())?;

        if sale_data.status == SaleStatus::Complete || sale_data.status == SaleStatus::Init {
            return Err("sale either complete or not inintialized".to_string());    
        }
        
        //new token id
        // let counter = canister_data_ref.collection_data.total_supply;

        // let collection_data = COLLECTION_DATA.with(|coll_data| { 
        //     coll_data.borrow().to_owned() }); 

        //mint function
        let symbol = canister_data_ref.collection_data.name.clone() +  &token_id;
        let uri = String::from("image url");
        let mint_res = mint(token_id, symbol, uri, buyer_id, 1);

        sale_data.status = SaleStatus::Complete;

        canister_data_ref.sales_data = sales_data;
        *canister_data.borrow_mut() = canister_data_ref;
        return mint_res;
    })
}

#[query] 
fn get_total_invested() -> u64 {

    let total_invest = CANISTER_DATA.with(|canister_data: &RefCell<CanisterData>| { 
        canister_data.borrow().total_invested });

    total_invest
}

#[query] 
fn get_sale_data(token_id : String) -> Result<SaleData, String> {

    CANISTER_DATA.with(|canister_data| {
        
        let canister_data_ref= canister_data.borrow().to_owned();
        let sales_data_map = canister_data_ref.sales_data;

        let sales_data = sales_data_map.get(&token_id).ok_or("invalid token_id")?;

        Ok(sales_data.clone())
    })
}

ic_cdk::export_candid!();

fn allow_only_canister() -> Result<(), String> {
    let canister_id = ic_cdk::api::id();
    if caller() != canister_id {
        Err(String::from("Access denied"))
    } else {
        Ok(())
    }
}
