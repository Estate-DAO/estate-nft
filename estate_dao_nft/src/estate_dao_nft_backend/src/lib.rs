mod state;

use candid::{types::number::Nat, CandidType, Deserialize, Principal};
use ic_cdk::api::time;
use state::{AdditionalMetadata, CollectionMetadata, FinancialDetails, MarketDetails, Metadata, NFTMetadata, PropertyDetails, SaleData, SaleStatus, Status};
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
type CollectionList = BTreeMap<String, CollectionMetadata>;
type TokenOwnerMap = BTreeMap<String, Principal>;
type UserTokensList = BTreeMap<Principal, Vec<String>>;
type SaleList = BTreeMap<String, SaleData>;

thread_local! {
    static NFT_STORE: RefCell<NFTList> = RefCell::default();
    static COLLECTION_DATA: RefCell<CollectionMetadata> = RefCell::default();
    static COUNTER: RefCell<u16> = RefCell::new (0u16);
    static COLLECTION_ID: RefCell<String> = RefCell::new(String::default());
    static TOKEN_OWNER: RefCell<TokenOwnerMap> = RefCell::default();
    static TOKEN_LIST: RefCell<UserTokensList> = RefCell::default();
    static SALE_DATA: RefCell<SaleList> = RefCell::default();
    static TOTOAL_INVESTED: RefCell<u64> = RefCell::new (0u64);
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

        let col_data = coll_data.borrow().to_owned();

        if col_data.is_initialised == true {
            // return Err("collection owner not initialized".to_string());
            Err("collection already initialised".to_string())
        }

        else {
            *coll_data.borrow_mut() = 
                CollectionMetadata{
                    collection_id:"1".to_string(), //remove
                    name,
                    desc,
                    total_supply: total_minted,
                    supply_cap:10000u16,
                    property_images:Vec::new(),
                    additional_metadata: Some(AdditionalMetadata{property_details:None, financial_details:None, documents:Vec::new(), market_details:None}),
                    status: Status::Draft,
                    owner: Principal::to_text(&owner),
                    is_initialised: true
            };
            COLLECTION_ID.with(|coll_id| {
                *coll_id.borrow_mut() = "1".to_string();
            });

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

    COLLECTION_DATA.with(|coll_data| {
        
        let mut col_data= coll_data.borrow_mut().to_owned();

        let user_res = Principal::from_text(col_data.owner.clone());
        let user: Principal;
        match user_res{
            Ok(id) => {user=id;},
            Err(e) => return Err("collection owner not initialized".to_string())
        };
        if caller() != user {
            return Err("unathorized user".to_string());
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

            *coll_data.borrow_mut() = col_data;

            return Ok("financial details added succesfully".to_string());
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
        
        let user_res = Principal::from_text(col_data.owner.clone());
        let user: Principal;
        match user_res{
            Ok(id) => {user=id;},
            Err(e) => return Err("collection owner not initialized".to_string())
        };
        if caller() != user {
            return Err("unathorized user".to_string());
        }
        else {
            let mut add_meta = col_data.additional_metadata.ok_or("collection not initialized")?;

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

            *coll_data.borrow_mut() = col_data;

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

    COLLECTION_DATA.with(|coll_data| {

        let mut col_data= coll_data.borrow_mut().to_owned();

        let user_res = Principal::from_text(col_data.owner.clone());
        let user: Principal;
        match user_res{
            Ok(id) => {user=id;},
            Err(e) => return Err("collection owner not initialized".to_string())
        };
        if caller() != user {
            return Err("unathorized user".to_string());
        }
        else {
            let mut add_meta = col_data.additional_metadata.ok_or("collection not initialized")?;

            add_meta.property_details = Some(add_det);
            col_data.additional_metadata = Some(add_meta);

            *coll_data.borrow_mut() = col_data;

            return Ok("additional details succesfully".to_string());
        }
    })
}

//collection specific data
// #[update(guard = "allow_only_authorized_principal")] 
// #[update] 
// fn init_additional_metadata( 
//     add_metadata: AdditionalMetadata,
//     prop_details: PropDetails
// ) -> Result<String, String> {

//     COLLECTION_DATA.with(|coll_data| {

//         let mut col_data= coll_data.borrow_mut().to_owned();

//         if col_data.additional_metadata.is_some() {
//             return Err("additional details already added".to_string());
//         }
//         if col_data.prop_details.is_some() {
//             return Err("property details already added".to_string());
//         }
//         col_data.additional_metadata = Some(add_metadata);

//         col_data.prop_details = Some(prop_details);

//         *coll_data.borrow_mut() = col_data;

//         //remove
//         let _collection_data_test = COLLECTION_DATA.with(|coll_data| { 
//             coll_data.borrow().to_owned() }); 

//         return Ok("collection created succesfully".to_string());

//     })
// }


// //collection specific data
// // #[update(guard = "allow_only_authorized_principal")] 
#[update] 
fn update_doc_details(
    doc_details: Vec<HashMap<String, String>> //errorEnum 
) -> Result<String, String> {

    COLLECTION_DATA.with(|coll_data| {

        let mut col_data= coll_data.borrow_mut().to_owned();

        let user_res = Principal::from_text(col_data.owner.clone());
        let user: Principal;
        match user_res{
            Ok(id) => {user=id;},
            Err(e) => return Err("collection owner not initialized".to_string())
        };
        if caller() != user {
            return Err("unathorized user".to_string());
        }
        else {
            
            let mut add_meta = col_data.additional_metadata.ok_or("collection not initialized")?;

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

// market details
#[query] 
fn get_market_details() -> Result<FinancialDetails, String> {

    let collection_data: CollectionMetadata = COLLECTION_DATA.with(|coll_data| { 
        coll_data.borrow().to_owned() });

    return Ok(collection_data.additional_metadata.ok_or("collection not initialized")?.financial_details.ok_or("collection not initialized")?);
}

// financial details
#[query] 
fn get_financial_details() -> Result<MarketDetails, String> {

    let collection_data: CollectionMetadata = COLLECTION_DATA.with(|coll_data| { 
        coll_data.borrow().to_owned() });

    return Ok(collection_data.additional_metadata.ok_or("collection not initialized")?.market_details.ok_or("collection not initialized")?);
}

// additional details
#[query] 
fn get_property_details() -> Result<PropertyDetails, String> {

    let collection_data: CollectionMetadata = COLLECTION_DATA.with(|coll_data| { 
        coll_data.borrow().to_owned() });

    return Ok(collection_data.additional_metadata.ok_or("collection not initialized")?.property_details.ok_or("collection not initialized")?);
}

// TODO update total_minted in collectionMetadata
#[update(guard = "allow_only_canister")] 
fn mint(token_id: String, symbol: String, uri: String, owner: Principal) -> Result<String, String> {

    let counter = COUNTER.with(|counter| {
        // *counter.borrow_mut() += 1;
        *counter.borrow()
    });

    let collection_id = COLLECTION_ID.with(|coll_id| (
            *coll_id.borrow()).clone());

    COLLECTION_DATA.with(|coll_data| {
        let mut col_data: CollectionMetadata= coll_data.borrow_mut().to_owned();
        col_data.total_supply = counter;
    });

    TOKEN_OWNER.with(|token_owner_map| {
        let mut owner_list =  token_owner_map.borrow_mut();
        owner_list.insert(token_id.clone(), owner.clone());
    });    
    
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

    NFT_STORE.with(|nft_list| {
        nft_list.borrow_mut().insert(
            token_id.clone(),
            NFTMetadata{
                collection_id: collection_id.to_string(),
                nft_symbol: symbol,
                nft_token_id: token_id,
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

        let nft_data = nft_data.ok_or("NFT not forund")?.to_owned();

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

#[update]
fn get_owner_of_NFT(token_id: String) -> Result<Principal, String> {

    TOKEN_OWNER.with(|token_owner| {
        let binding = token_owner.borrow().to_owned();
        let token_owner_map = binding.get(&token_id);

        return Ok(*token_owner_map.ok_or("invalid token_id")?);
    })
}

#[update]
fn add_collection_image(image: String) -> Result<String, String> {

    COLLECTION_DATA.with(|coll_data| {
        
        let mut col_data= coll_data.borrow_mut().to_owned();
        col_data.property_images.push(image);

        *coll_data.borrow_mut() = col_data;

        return Ok("sucess".to_string());
    })
}

#[query] 
fn collection_image() -> Vec<String>{

    let collection_data = COLLECTION_DATA.with(|coll_data| { 
        coll_data.borrow().to_owned() });
      
    collection_data.property_images
}


#[query(composite = true)]
async fn create_accountid(caller_account: Principal) -> Result<AccountIdentifier, String> {
    let canister_id = ic_cdk::api::id();

    let ledger_canister_id = Principal::from_text("ryjl3-tyaaa-aaaaa-aaaba-cai").unwrap();
    // let account = AccountIdentifier::new(&caller_account, &principal_to_subaccount(&caller_account));
    let account = AccountIdentifier::new(&caller_account, &DEFAULT_SUBACCOUNT);

    let balance_args = ic_ledger_types::AccountBalanceArgs { account };
    Ok(account)

}


#[update]
async fn delegate_transfer(receiver_account: Principal, owner_id: Principal) -> Result<Nat, String> {
    let canister_id = ic_cdk::api::id();

    let ledger_canister_id = Principal::from_text("ryjl3-tyaaa-aaaaa-aaaba-cai").unwrap();
    // let account = AccountIdentifier::new(&caller_account, &principal_to_subaccount(&caller_account));
    let receive_account = AccountIdentifier::new(&receiver_account, &DEFAULT_SUBACCOUNT);
    let canister_account = AccountIdentifier::new(&canister_id, &DEFAULT_SUBACCOUNT);
    let sender_account = AccountIdentifier::new(&owner_id, &DEFAULT_SUBACCOUNT);
    // todo check allowance

    let transfer_from_args = TransferFromArgs {
        spender_subaccount: None,
        memo: None,
        amount: Nat::from(2_000_000u64),
        fee: None,
        from: Account::from(owner_id),
        to: Account::from(receiver_account),
        created_at_time: None,
    };
    // icrc_ledger_types::icrc2::transfer_from(ledger_canister_id, transfer_args)
    //     .await
    //     .unwrap().unwrap();

    let res =  call(ledger_canister_id, "icrc2_transfer_from", (transfer_from_args,), ).await;
    match res{
        Ok(r) => {
            let (res,): (Result<Nat, TransferFromError>,) = r;
            Ok(res.unwrap())
        },
        Err(e) => Err(e.1)
    }
}

#[update]
async fn primary_sale(receiver_id: Principal, buyer_id: Principal) -> Result<String, String> {
    let canister_id = ic_cdk::api::id();

    let collection_data = COLLECTION_DATA.with(|coll_data| { 
        coll_data.borrow().to_owned() }); 

    let ledger_canister_id = Principal::from_text("ryjl3-tyaaa-aaaaa-aaaba-cai").unwrap();
    // let account = AccountIdentifier::new(&caller_account, &principal_to_subaccount(&caller_account));
    // let receive_account = AccountIdentifier::new(&receiver_account, &DEFAULT_SUBACCOUNT);
    // let canister_account = AccountIdentifier::new(&canister_id, &DEFAULT_SUBACCOUNT);
    // let buyer_account: AccountIdentifier = AccountIdentifier::new(&buyer_id, &DEFAULT_SUBACCOUNT);
    
    // new token_id
    let token_counter = COUNTER.with(|counter| {
    //     *counter.borrow_mut() += 1;
        *counter.borrow()
    });

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
        Err(e) => todo!()
    }
    let nft_price = collection_data.additional_metadata
        .ok_or("collection additional metadata not initalized")?
        .financial_details.ok_or("unable to fetch prop price".to_string())?
        .investment.ok_or("unable to fetch investment details")?
        .min_investment.ok_or("unable to fetch selling price details")?;

    // let receiver_account = collection_data.owner;
    // let receiver_id =  Principal::from_text(receiver_account).unwrap();
    // let nft_price = 1000000u64;
    // check if allowance > nft price
    if delegated_amount.allowance < nft_price {
        return Err("delegated amount less than NFT price".to_string());
    }

    let mut sale_data = SaleData{
        nft_token_id: ((token_counter + 1).to_string()),
        buyer: buyer_id,
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
                Err(e) => {transfer_status = 0}
            }
        },
        Err(e) => {transfer_status = 0}
    }

    if transfer_status == 0{
        return Err("error transfering from canister to seller".to_string());
    }

    //update total_investes field
    TOTOAL_INVESTED.with(|total_invest| {
        *total_invest.borrow_mut() += nft_price;
    });

    let counter = COUNTER.with(|counter| {
        *counter.borrow_mut() += 1;
        *counter.borrow()
    });

    //tbd symbol, uri

    //mint function
    let symbol = collection_data.name +  &counter.to_string();
    let uri = String::from("image url");
    let mint_res = mint(counter.to_string(), symbol, uri, buyer_id);

    match mint_res {
        Ok(r) => {
            sale_data.status = SaleStatus::Complete;

            SALE_DATA.with(|sale_list_map| {
                let mut owner_list =  sale_list_map.borrow_mut();
                owner_list.insert(counter.to_string(), sale_data);
            });
            Ok(r)
        },
        Err(e) => {
            sale_data.status = SaleStatus::Incomplete;

            SALE_DATA.with(|sale_list_map| {
                let mut sale_list =  sale_list_map.borrow_mut();
                sale_list.insert(counter.to_string(), sale_data);
            });
            Err(e)
        },
    }

}


#[update] 
fn primary_sale_mint(token_id : String) -> Result<String, String> {

    let buyer_id = caller();
    SALE_DATA.with(|sale_list_map| {
        let mut sale_list = sale_list_map.borrow_mut();
        let mut sale_data = sale_list.get_mut(&token_id).ok_or("unable fetch no sale exist for this token".to_string())?;


        if sale_data.status == SaleStatus::Complete || sale_data.status == SaleStatus::Init {
            return Err("sale either complete or not inintialized".to_string());    
        }
        else{
            //new token id
            let counter = COUNTER.with(|counter| {
                *counter.borrow()
            });

            let collection_data = COLLECTION_DATA.with(|coll_data| { 
                coll_data.borrow().to_owned() }); 
        

            //mint function
            let symbol = collection_data.name +  &counter.to_string();
            let uri = String::from("image url");
            let mint_res = mint(token_id, symbol, uri, buyer_id);

            sale_data.status = SaleStatus::Complete;

            return mint_res;
        }
    })
}

#[query] 
fn get_total_invested() -> u64 {

    let total_invested = TOTOAL_INVESTED.with(|total_invest| {
        *total_invest.borrow()
    });

    total_invested
}

#[query] 
fn get_sale_data(token_id : String) -> Result<SaleData, String> {

    SALE_DATA.with(|sale_list| {
        let sale_lists = sale_list.borrow();
        let sale_data = sale_lists.get(&token_id);

        let sale_data = sale_data.ok_or("SALE not forund")?.to_owned();

        return Ok(sale_data);
    }) 
}

ic_cdk::export_candid!();

/////////////////////
// fn allow_only_authorized_principal() -> Result<(), String> {
//     let authorized_principal_id = Principal::from_text("2ghx4-leaaa-aaaaa-qacru-cai").unwrap();
//     if caller() != authorized_principal_id {
//         Err(String::from("Access denied"))
//     } else {
//         Ok(())
//     }
// }


fn allow_only_canister() -> Result<(), String> {
    
    let canister_id = ic_cdk::api::id();

    if caller() != canister_id {
        Err(String::from("Access denied"))
    } else {
        Ok(())
    }
}