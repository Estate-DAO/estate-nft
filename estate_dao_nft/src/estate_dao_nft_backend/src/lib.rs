mod state;

use candid::types::principal;
use candid::{types::number::Nat, CandidType, Deserialize, Principal};
use ic_cdk::api::{is_controller, time};
use ic_cdk::storage;
use icrc_ledger_types::icrc2::approve;
// use icrc_ledger_types::icrc1::transfer::TransferError;
// use icrc_ledger_types::icrc1::account::Subaccount;
use state::{AdditionalMetadata, CollectionMetadata, FinancialDetails, FormMetadata, MarketDetails, Metadata, NFTMetadata, PropertyDetails, SaleData, SaleStatus, Status};
use serde::Serialize;

use ic_cdk::{api::call::call, query, update, init, caller};
use std::collections::btree_map::Keys;
use std::ops::Index;
use std::{clone, string};
use std::{cell::RefCell};
use std::collections::{BTreeMap, HashMap};

use ic_ledger_types::{
    AccountBalanceArgs, AccountIdentifier, Memo, Subaccount, Tokens, TransferArgs, DEFAULT_SUBACCOUNT, TransferError
};
use icrc_ledger_types::{
    icrc1::account::Account, 
    icrc2::{
        allowance::{Allowance, AllowanceArgs},
        transfer_from::{TransferFromArgs, TransferFromError}
    }
};

// TODO: store balance in u128
// add wrapper from u64 -> u128
// all inputs in ICP not e8s

pub const LEDGER_CANISTER_ID: &str = "ryjl3-tyaaa-aaaaa-aaaba-cai";
const ICP_FEE: u64 = 10_000;

type NFTList = BTreeMap<String, NFTMetadata>;
type TokenOwnerMap = BTreeMap<String, Account>;
type UserTokensList = BTreeMap<Account, Vec<String>>;
type SaleList = BTreeMap<String, SaleData>;
type CumulativeTotalInvestmentInCollection =  u64;
// type CumulativeTotalInvestmentInCollection =  u64;

//todo remove used_balance
type UserBalance = BTreeMap<Principal, (CumulativeTotalInvestmentInCollection, u64)>;
type UserPayAccount = BTreeMap<Principal, Principal>;

#[derive(Clone, Debug, CandidType, Default, Deserialize, Serialize)]
pub struct CanisterData { 
    pub collection_data: CollectionMetadata,
    pub nft_store: NFTList,
    pub token_owner: TokenOwnerMap,
    pub sales_data: SaleList,
    pub total_invested: u64,
    pub user_balance: UserBalance,
    pub user_pay_account: UserPayAccount,
    pub sale_refund_reprocess: Vec<Principal>,
    pub sale_mint_reprocess: Vec<Principal>,
    pub sale_transfer_reprocess: Vec<Principal>,

}

thread_local! {
    static CANISTER_DATA: RefCell<CanisterData> = RefCell::default()
}

//collection specific data
#[update]
fn init_collection(
    form_data: FormMetadata
) -> Result<String, String> {

    if !is_controller(&caller()) {
        return Err("UnAuthorised Access".to_string());
    }

    CANISTER_DATA.with(|canister_data| {

        // let total_minted = COUNTER.with(|counter| *counter.borrow());
        let mut canister_data_ref = canister_data.borrow_mut();


        if canister_data_ref.collection_data.is_initialised == true {
            return Err("collection already initialised".to_string());
        }

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
                // symbol: form_data.symbol,
                total_supply: 0,
                price: form_data.price,
                image_uri: form_data.image_uri,
                supply_cap: form_data.supply_cap,
                // image: Some("image".to_string()),
                property_images: form_data.property_images,
                additional_metadata: add_metadata,
                status: Status::Live,
                owner: form_data.owner,
                treasury_account: form_data.treasury,
                is_initialised: true,
                primary_sale_happened: false
        };

        Ok("collection created succesfully".to_string())
    })
}

// tbd

// #[update]
// fn update_basic_details( 
//     name: Option<String>,
//     desc: Option<String>,
//     stat: Option<Status>
// ) -> Result<String, String> {

//     CANISTER_DATA.with(|canister_data| {
        
//         let mut canister_data_ref= canister_data.borrow_mut().to_owned();
//         let mut col_data = canister_data_ref.collection_data;

//         let user_res = Principal::from_text(col_data.owner.clone());
//         let user: Principal;
//         match user_res{
//             Ok(id) => {user=id;},
//             Err(e) => return Err("collection owner not initialized".to_string())
//         };
//         if caller() != user {
//             Err("unathorized user".to_string())
//         }
//         else {
//             match name{
//                 Some(new_name) => {
//                     col_data.name = new_name;
//                 }
//                 _ => {}
//             }            
//             match desc{
//                 Some(new_description) => {
//                     col_data.desc = new_description;
//                 }
//                 _ => {}
//             }            
//             match stat{
//                 Some(new_status) => {
//                     col_data.status = new_status;
//                 }
//                 _ => {}
//             }

//             canister_data_ref.collection_data = col_data;
//             *canister_data.borrow_mut() = canister_data_ref;

//             Ok("basic details updated succesfully".to_string())
//         }
//     })
// }

//update status of collection
//only controller and canister itself can update the status
#[update]
fn update_status( 
    new_status: Status
) -> Result<String, String> {

    let canister_id = ic_cdk::api::id();
    let caller_account = caller();

    if caller_account == canister_id || is_controller(&caller()) {
    
        CANISTER_DATA.with_borrow_mut(|canister_data| {
            
            canister_data.collection_data.status = new_status;
        }); 
        
        Ok("basic details updated succesfully".to_string())
    }
    else{
        Err("Unauthorized access".to_string())
    }

}

// // #[update(guard = "allow_only_authorized_principal")] 
#[update] 
fn update_market_details( 
    market_det: MarketDetails
) -> Result<String, String> {

    CANISTER_DATA.with_borrow_mut(|canister_data| {
        
        // let mut col_data = canister_data.collection_data;

        let user_res = Principal::from_text(canister_data.collection_data.owner.clone());
        let user: Principal;
        match user_res{
            Ok(id) => {user=id;},
            Err(_e) => return Err("collection owner not initialized".to_string())
        };
        if caller() != user {
            Err("unathorized user".to_string())
        }
        else {
            let mut add_meta = &mut canister_data.collection_data.additional_metadata;
            match add_meta {
                Some(val) => {
                    val.market_details = Some(market_det);
                }
                None => {
                    return Err("additional metadata not initialized".to_string());
                }
            }

            Ok("market details added succesfully".to_string())
        }
    })
}

// // #[update(guard = "allow_only_authorized_principal")] 
#[update] 
fn update_financial_details( 
    financial_det: FinancialDetails
) -> Result<String, String> {

    CANISTER_DATA.with_borrow_mut(|canister_data| {
        
        // let mut col_data = canister_data.collection_data;

        let user_res = Principal::from_text(canister_data.collection_data.owner.clone());
        let user: Principal;
        match user_res{
            Ok(id) => {user=id;},
            Err(_e) => return Err("collection owner not initialized".to_string())
        };
        if caller() != user {
            Err("unathorized user".to_string())
        }
        else {
            let mut add_meta = &mut canister_data.collection_data.additional_metadata;
            match add_meta {
                Some(val) => {
                    val.financial_details = Some(financial_det);
                }
                None => {
                    return Err("additional metadata not initialized".to_string());
                }
            }
            Ok("financial details added succesfully".to_string())
        }
    })
}


// //collection specific data
// // #[update(guard = "allow_only_authorized_principal")] 
#[update] 
fn update_property_details( 
    add_det: PropertyDetails
) -> Result<String, String> { 
    CANISTER_DATA.with_borrow_mut(|canister_data| {
        
        let user_res = Principal::from_text(canister_data.collection_data.owner.clone());
        let user: Principal;
        match user_res{
            Ok(id) => {user=id;},
            Err(_e) => return Err("collection owner not initialized".to_string())
        };
        if caller() != user {
            Err("unathorized user".to_string())
        }
        else {
            let mut add_meta = &mut canister_data.collection_data.additional_metadata;
            match add_meta {
                Some(val) => {
                    val.property_details = Some(add_det);
                }
                None => {
                    return Err("additional metadata not initialized".to_string());
                }
            }

            Ok("property details added succesfully".to_string())
        }
    })
}


// //collection specific data
// // #[update(guard = "allow_only_authorized_principal")] 
// #[update] 
// fn update_doc_details(
//     doc_details: Vec<HashMap<String, String>> //errorEnum 
// ) -> Result<String, String> {

//     CANISTER_DATA.with_borrow_mut(|canister_data| {
        
//         let mut col_data = canister_data.collection_data;

//         let user_res = Principal::from_text(col_data.owner.clone());
//         let user: Principal;
//         match user_res{
//             Ok(id) => {user=id;},
//             Err(e) => return Err("collection owner not initialized".to_string())
//         };
//         if caller() != user {
//             Err("unathorized user".to_string())
//         }
//         else {
            
//             let mut add_meta = col_data.additional_metadata.ok_or("collection not initialized")?;

//             add_meta.documents = doc_details;
//             col_data.additional_metadata = Some(add_meta);

//             canister_data.collection_data = col_data;

//             Ok("Documents added succesfully".to_string())
//         }
//     })
// }

#[query] 
fn icrc7_name() -> String {

    CANISTER_DATA.with(|canister_data| { 
        let name = canister_data.borrow().collection_data.name.clone();
    
        name
    })
}

#[query] 
fn icrc7_image() -> String {

    let collection_data = CANISTER_DATA.with(|canister_data| { 
        canister_data.borrow().collection_data.to_owned() });

    collection_data.image_uri
}

#[query] 
fn icrc7_description() -> String {

    let collection_data = CANISTER_DATA.with(|canister_data| { 
        canister_data.borrow().collection_data.to_owned() });

    collection_data.desc
}

#[query] 
fn icrc7_total_supply() -> u64 {

    let collection_data = CANISTER_DATA.with(|canister_data| { 
        canister_data.borrow().collection_data.to_owned() });

    collection_data.total_supply
}

#[query] 
fn get_collection_metadata() -> Result<CollectionMetadata, String> {

    let collection_data = CANISTER_DATA.with(|canister_data: &RefCell<CanisterData>| { 
        canister_data.borrow().collection_data.to_owned() });

    Ok(collection_data)
}

#[query] 
fn get_collection_status() -> Result<Status, String> {

    let collection_data = CANISTER_DATA.with(|canister_data| { 
        canister_data.borrow().collection_data.to_owned() });

    Ok(collection_data.status)
}

// financial details
#[query] 
fn get_financial_details() -> Result<FinancialDetails, String> {

    let collection_data = CANISTER_DATA.with(|canister_data| { 
        canister_data.borrow().collection_data.to_owned() });

    Ok(collection_data.additional_metadata.ok_or("collection not initialized")?.financial_details.ok_or("collection not initialized")?)
}

// market details
#[query] 
fn get_market_details() -> Result<MarketDetails, String> {

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


// mint new NFTs
#[update(guard = "allow_only_canister")] 
fn mint(token_id: String, symbol: String, uri: String, owner: Principal) -> Result<String, String> {

    CANISTER_DATA.with_borrow_mut(|canister_data| {

        let owner_account = Account::from(owner);
        canister_data.token_owner.insert(token_id.clone(), owner_account);

        canister_data.nft_store.insert(
                token_id.clone(),
                NFTMetadata{
                    nft_symbol: symbol,
                    nft_token_id: token_id,
                    nft_uri: uri
                }
            );
    
        Ok("NFT minted succesfully".to_string())
    })

}

// Metadata items TBD
//using NFTMetadata + Collection Metadata 
#[update] 
fn get_metadata(token_id : String) -> Result<Metadata, String> {

    CANISTER_DATA.with_borrow(|canister_data| {

        let nft_data = canister_data.nft_store.get(&token_id).cloned();
        let nft_data = nft_data.ok_or("NFT not found")?.to_owned();

        let metadata = Metadata{
            symbol: nft_data.nft_symbol,
            nft_token_id: nft_data.nft_token_id,
            nft_uri: nft_data.nft_uri,
            collection_name: canister_data.collection_data.name.clone(),
            desc: canister_data.collection_data.desc.clone(),
            total_supply: canister_data.collection_data.total_supply,
            supply_cap: canister_data.collection_data.supply_cap
        };
    Ok(metadata)
    }) 
}

#[update]
fn icrc7_owner_of(token_id: String) -> Result<Account, String> {

    CANISTER_DATA.with_borrow(|canister_data| {
        
        let token_owner = canister_data.token_owner.get(&token_id).cloned();

        token_owner.ok_or("invalid token_id".to_string())
    })
}

#[update]
fn add_collection_image(image: String) -> Result<String, String> {

    CANISTER_DATA.with_borrow_mut(|canister_data| {

        canister_data.collection_data.property_images.push(image);

        Ok("sucess".to_string())
    })
}

#[query] 
fn get_collection_image() -> Vec<String>{

    CANISTER_DATA.with_borrow(|canister_data| { 
        canister_data.collection_data.property_images.clone()
        
    })
  
}

#[update] 
fn update_NNS_account( 
    user_nns_account: Principal,
) -> Result<String, String> {

    let caller = get_caller().expect("Anonymus principal not allowed to make calls");
    // let caller = caller();

    // let user_pay_account = 
    CANISTER_DATA.with(|canister_data| {   
        let mut canister_data_ref =  canister_data.borrow().to_owned();

        let user_payment_account = canister_data_ref.user_pay_account.get(&caller);       
        match user_payment_account {
            Some(_val) => {
                Err("account already added".to_string())
            },
            None => {
                canister_data_ref.user_pay_account.insert(caller, user_nns_account);
                *canister_data.borrow_mut() = canister_data_ref;
                Ok("account added successfully".to_string())
            }
        }
    })
}

#[query] 
fn get_NNS_account( 
) -> Result<Principal, String> {

    let caller = get_caller().expect("Anonymus principal not allowed to make calls");
    // let caller = caller();

    // let user_pay_account = 
    let canister_data_ref = CANISTER_DATA.with(|canister_data| { 
        canister_data.borrow().to_owned() });

    let user_payment_account = canister_data_ref.user_pay_account.get(&caller);       
    match user_payment_account {
        Some(nns_account) => {
            Ok(*nns_account)
        },
        None => {
            Err("account not added".to_string())
        }
    }
}


#[update]
async fn primary_sale() -> Result<String, String> {
    let caller = caller();
    let canister_id = ic_cdk::api::id();

    let user = CANISTER_DATA.with_borrow(|canister_data| { 
        canister_data.user_pay_account.get(&caller).cloned()
    }).ok_or("nns account not added for user")?;
    
    let ledger_canister_id = Principal::from_text(LEDGER_CANISTER_ID).unwrap();

    let nft_price = CANISTER_DATA.with_borrow(|canister_data| { 
        canister_data.collection_data.price });

    let account = AccountIdentifier::new(&canister_id, &Subaccount::from(user));

    let balance_args = ic_ledger_types::AccountBalanceArgs { account };
    let balance = ic_ledger_types::account_balance(ledger_canister_id, balance_args)
        .await
        .map_err(|e| e.1);

    let token_balance: Tokens;
    match balance{
        Ok(tokens) => {
            token_balance = tokens;
        },
        Err(e) => return Err(e),
    };
        
    let current_balance = token_balance.e8s();

    //fetch stored_balance
    let user_balance = CANISTER_DATA.with(|canister_data| { 
        canister_data.borrow().user_balance.get(&user).cloned() });

    match  user_balance{
        Some((stored_bal, used_bal)) => {
            if stored_bal < current_balance {

                CANISTER_DATA.with_borrow_mut(|canister_data| {
                    canister_data.user_balance.insert(user, (current_balance, used_bal));
                    
                    canister_data.total_invested = canister_data.total_invested.saturating_add(current_balance.saturating_sub(stored_bal));
                    
                });
                return Ok("balance_updated".to_string());
            }
            return  Err("no new transfer".to_string());
        }   
        None => {
            if current_balance >= nft_price {
                CANISTER_DATA.with_borrow_mut(|canister_data| {
                    canister_data.user_balance.insert(user, (current_balance, 0));
                    canister_data.total_invested = canister_data.total_invested.saturating_add(current_balance);
                });
                return Ok("balance updated for new user".to_string());
            } 
            return  Err("no transfered or amount less than NFT price".to_string());
        },
    }
}

//mint for individual user, to be minted comparing the stored amount of user, and price of indivvidual NFT
#[update(guard = "allow_only_canister")] 
fn mint_approved_nfts(user_account: Principal) -> Result<String, String> {
    let canister_id = ic_cdk::api::id();
    // let mut counter: u16 = Default::default();    

    let canister_data_ref = CANISTER_DATA.with(|canister_data| { 
        canister_data.borrow().to_owned() });

    let nft_price = canister_data_ref.collection_data.price;

    let total_minted_nfts =  canister_data_ref.collection_data.total_supply;
    let mut counter = total_minted_nfts.saturating_add(1);

    let mut mint_allowance: u64;

    //  check user data for 
    let user_data = canister_data_ref.user_balance.to_owned();
    let user_balance = user_data.get(&user_account);

    match  user_balance{
        Some((stored_bal, used_bal)) => {

            mint_allowance = (stored_bal.saturating_sub(*used_bal)).saturating_div(nft_price);
            ic_cdk::println!("mint allowance 22: {}", mint_allowance);
        }
        _ => {
            return Err("no balance found for user".to_string());
        }  
    }

    let uri = canister_data_ref.collection_data.image_uri;

    //check for approved mints remaining    
    // let approved_mints = 5;

    for _mints in 0 .. mint_allowance{
        let symbol = canister_data_ref.collection_data.name.clone() + "_" + &counter.to_string();

        let collection_data_ref = CANISTER_DATA.with(|canister_data| { 
            canister_data.borrow().collection_data.to_owned()});

        if collection_data_ref.total_supply >= collection_data_ref.supply_cap {
            return Err("supply cap limit reached".to_string());
        }    

        CANISTER_DATA.with(|canister_data| {

            let mut canister_data_ref = canister_data.borrow().to_owned();
            // canister_data_ref.total_invested = canister_data_ref.total_invested.saturating_add(nft_price);
            canister_data_ref.collection_data.total_supply = canister_data_ref.collection_data.total_supply.saturating_add(1);

            counter = canister_data_ref.collection_data.total_supply;

            //update user balance
            let mut user_stored_bal = canister_data_ref.user_balance.get(&user_account).unwrap().to_owned();

            user_stored_bal.1 = user_stored_bal.1.saturating_add(nft_price);
            canister_data_ref.user_balance.insert(user_account, user_stored_bal);

            //add sales data
            let sale_data = SaleData{
                nft_token_id: counter.to_string(),
                buyer: Account::from(user_account),    
                amount: nft_price,
                status: SaleStatus::Complete,
                time: ic_ledger_types::Timestamp { timestamp_nanos: time() }
            };
            canister_data_ref.sales_data.insert(counter.to_string(), sale_data);

            *canister_data.borrow_mut() = canister_data_ref;
        });   
        let _mint_res = mint(counter.to_string(), symbol.clone(), uri.clone(), user_account);
    }

    Ok("success".to_string())
}

#[update(guard = "allow_only_canister")] 
fn sale_confirmed_mint() -> Result<String, String> {

    let canister_data_ref = CANISTER_DATA.with(|canister_data| { 
        canister_data.borrow().to_owned() });

    let user_balance = canister_data_ref.user_balance;
    for (key, _value) in user_balance.iter() {
        let res = mint_approved_nfts(*key);
        match res {
            Ok(_val) => {continue;},
            Err(_error_str) => { 
                CANISTER_DATA.with(|canister_data: &RefCell<CanisterData>| {
                    let mut canister_data_ref= canister_data.borrow().to_owned();
                    canister_data_ref.sale_mint_reprocess.push(*key);
                    *canister_data.borrow_mut() = canister_data_ref;
                });
            }
        }
    }
    Ok("NFTs minted succesfully for all participants".to_string())
}

#[query]
fn get_payment_details() -> Result<(String, u64, u64), String> {
    let canister_id = ic_cdk::api::id();
    // let caller = get_caller().expect("Anonymus principal not allowed to make calls");
    let caller = caller();

    CANISTER_DATA.with_borrow(|canister_data| { 

        // nft price
        let nft_price = canister_data.collection_data.price;

        // account-id
        let nns_payment_account_principal = canister_data.user_pay_account.get(&caller).ok_or("nns account not added for user")?;
        let account_id = AccountIdentifier::new(&canister_id, &Subaccount::from(*nns_payment_account_principal));

        //user's invested amount
        let user_balance = canister_data.user_balance.get(&nns_payment_account_principal);
        let user_stored_balance = match  user_balance{
            Some((stored_bal, _used_bal)) => {
                *stored_bal
            }   
            None => {0},
        };
        Ok((account_id.to_string(), nft_price, user_stored_balance))
    })
}


#[update]
async fn get_balance(user_account: Principal) -> Result<u64, String> {
    let canister_id = ic_cdk::api::id();

    let ledger_canister_id = Principal::from_text(LEDGER_CANISTER_ID).unwrap();
    let account = AccountIdentifier::new(&canister_id, &Subaccount::from(user_account));

    let balance_args = ic_ledger_types::AccountBalanceArgs { account };
    // return Ok(balance_args)
    let balance = ic_ledger_types::account_balance(ledger_canister_id, balance_args)
        .await
        .map_err(|e| e.1);

    let token_balance: Tokens;
    match balance{
        Ok(tokens) => {
            token_balance = tokens;
        },
        Err(e) => return Err(e),
    }
   
    Ok(token_balance.e8s())
}

//refund escrow balance for a perticular user, when sale is rejected
#[update(guard = "allow_only_canister")] 
async fn refund_user_tokens(user : Principal) -> Result<String, String> {
    let canister_id = ic_cdk::api::id();

    let ledger_canister_id = Principal::from_text(LEDGER_CANISTER_ID).unwrap();
    let account = AccountIdentifier::new(&canister_id, &Subaccount::from(user));

    let balance_args = ic_ledger_types::AccountBalanceArgs { account };
    // return Ok(balance_args)
    let balance = ic_ledger_types::account_balance(ledger_canister_id, balance_args)
        .await
        .map_err(|e| e.1);

    let token_balance: Tokens;
    match balance{
        Ok(tokens) => {
            token_balance = tokens;
        },
        Err(e) => {
            return Err("unable to fetch balance".to_string());
        },
    }
    let escrow_token_balance = token_balance.e8s();
    if escrow_token_balance <= ICP_FEE{ 
        return Err("no balance for user".to_string());
    }

    let transfer_args = TransferArgs {
        memo: ic_ledger_types::Memo(0),
        amount: Tokens::from_e8s(escrow_token_balance.saturating_sub(ICP_FEE)),
        fee: Tokens::from_e8s(ICP_FEE),
        from_subaccount: Some(Subaccount::from(user)),
        to: AccountIdentifier::new(&user, &DEFAULT_SUBACCOUNT),
        created_at_time: None,
    };

    // TODO: handle error 
    //transfer function of ic_ledger_types
    ic_ledger_types::transfer(ledger_canister_id, transfer_args)
        .await
        .expect("call to ledger failed")
        .expect("transfer failed");

    // TODO: check balance of user account in block 
    //research

    //check for updated balance, while updating stored balance  
    //TODO: check replace of Btreemap

    CANISTER_DATA.with_borrow_mut(|canister_data| { 

        let user_balance = canister_data.user_balance.get(&user);
        match  user_balance{
            Some((_stored_bal, used_bal)) => {
                    CANISTER_DATA.with(|canister_data| {
                        let mut canister_data_ref= canister_data.borrow_mut();
                        canister_data_ref.user_balance.insert(user, (0, *used_bal));

                    });
            }   
            None => {
                return  Err("user had no balance".to_string());
            },
        }
        Ok("amount refunded to user".to_string())
    })

}

//refund balance for a perticular user
#[update]
async fn refund_for_user_by_controller(user : Principal) -> Result<String, String> {

    if !is_controller(&caller()) {
        return Err("UnAuthorised Access".into());
    }

    match refund_user_tokens(user).await {
        Ok(res) => {
            Ok(res)
        },
        Err(error_str) => {
            Err(error_str)
        }
    }
}

//rejects the sale, refunding all users 
#[update] 
async fn sale_rejected() -> Result<String, String> {

    if !is_controller(&caller()) {
        return Err("UnAuthorised Access".into());
    }

    let user_balance = CANISTER_DATA.with_borrow(|canister_data| { 
        canister_data.user_balance.to_owned() });

    for (key, _value) in user_balance.iter() {
        let res = refund_user_tokens(*key).await;
        match res {
            Ok(_val) => {continue;},
            Err(_error_str) => {
                CANISTER_DATA.with_borrow_mut(|canister_data| {
                    canister_data.sale_refund_reprocess.push(*key);
                });
            }
        }
    }

    let _update_status_res = update_status(Status::Refunded);
    Ok("Amount refunded succesfully for all participants".to_string())
}


//sale accepted, transfer funds to treasury
#[update(guard = "allow_only_canister")] 
async fn sale_confirmed_transfer() -> Result<String, String> {

    let user_balance = CANISTER_DATA.with_borrow(|canister_data| { 
        canister_data.user_balance.to_owned()});

    // let user_balance = canister_data_ref.user_balance;
    for (key, _value) in user_balance.iter() {
        let res = transfer_user_tokens(*key).await;
        match res {
            Ok(_val) => {continue;},
            Err(_error_str) => {
                CANISTER_DATA.with(|canister_data: &RefCell<CanisterData>| {
                    let mut canister_data_ref= canister_data.borrow().to_owned();
                    // let mut col_data = canister_data_ref.collection_data;
                    canister_data_ref.sale_transfer_reprocess.push(*key);
                    *canister_data.borrow_mut() = canister_data_ref;
                });
            }
        }
    }
    Ok("Amount transferred succesfully to treasury for all participants".to_string())
}

#[update(guard = "allow_only_canister")] 
async fn transfer_user_tokens(user : Principal) -> Result<String, String> {
    let canister_id = ic_cdk::api::id();

    let ledger_canister_id = Principal::from_text(LEDGER_CANISTER_ID).unwrap();
    let escrow_account = AccountIdentifier::new(&canister_id, &Subaccount::from(user));

    let balance_args = ic_ledger_types::AccountBalanceArgs {account: escrow_account };
    // return Ok(balance_args)
    let balance = ic_ledger_types::account_balance(ledger_canister_id, balance_args)
        .await
        .map_err(|e| e.1);

    let token_balance: Tokens;
    match balance{
        Ok(tokens) => {
            token_balance = tokens;
        },
        Err(e) => {
            return Err("unable to fetch balance".to_string());
        },
    }
    let escrow_token_balance = token_balance.e8s();
    if escrow_token_balance == 0 || escrow_token_balance < ICP_FEE{ 
        return Ok("no balance for user in escrow".to_string());
    }

    let collection_data = CANISTER_DATA.with(|canister_data| { 
        canister_data.borrow().collection_data.to_owned() });

    let treasury_id = collection_data.treasury_account;
    let nft_price = collection_data.price;
    
    let treasury_principal = Principal::from_text(treasury_id).unwrap();

    let treasury_account = AccountIdentifier::new(&treasury_principal, &DEFAULT_SUBACCOUNT);
    
    //fetch user store balance
    let user_data = CANISTER_DATA.with(|canister_data| { 
        canister_data.borrow().user_balance.to_owned() });

    let user_token_balance = user_data.get(&user).ok_or("user had no balance")?;
    let user_stored_balance = user_token_balance.0;
    let user_used_balance = user_token_balance.1;

    let excess_user_balance = user_stored_balance % nft_price;
    let transfer_balance = user_stored_balance.saturating_sub(excess_user_balance);

    let transfer_args = TransferArgs {
        memo: ic_ledger_types::Memo(0),
        amount: Tokens::from_e8s(transfer_balance.saturating_sub(ICP_FEE)),
        fee: Tokens::from_e8s(ICP_FEE),
        from_subaccount: Some(Subaccount::from(user)),
        to: treasury_account,
        created_at_time: None,
    };

    //transfer function of ic_ledger_types
    let _res = ic_ledger_types::transfer(ledger_canister_id, transfer_args)
        .await
        .map_err(|_| "call to ledger failed".to_string())?
        .map_err(|_| "call to ledger failed".to_string())?; 


    CANISTER_DATA.with(|canister_data| {
        let mut canister_data_ref= canister_data.borrow().to_owned();
        canister_data_ref.user_balance.insert(user, (excess_user_balance, user_used_balance));

        *canister_data.borrow_mut() = canister_data_ref;
    });

    Ok("success".to_string())

}

#[update] 
async fn sale_accepted() -> Result<String,String>{
    if !is_controller(&caller()) {
        return Err("UnAuthorised Access".into());
    }

    let primarys_sale_check = CANISTER_DATA.with(|canister_data| { 
        canister_data.borrow().collection_data.primary_sale_happened.to_owned() });
    if primarys_sale_check {
        return Err("primary sale already happened".to_string());
    }
    
    match sale_confirmed_mint() {
        Ok(str) => {
            ic_cdk::println!("mint result {:?}", str);
        },
        Err(e) => {
            return Err(e);
        }
    }
    match sale_confirmed_transfer().await {
        Ok(str) => {
            ic_cdk::println!("mint result {:?}", str);
        },
        Err(e) => {
            return Err(e);
        }
    }
    let _update_status_res = update_status(Status::Minted);
    CANISTER_DATA.with(|canister_data| { 

        let mut canister_data_ref = canister_data.borrow_mut();
        canister_data_ref.collection_data.primary_sale_happened = true;
    
        Ok("Sale accpted, NFTs minted, and amount transferred to treasury".to_string())
    })
}

// call failed mints and transfer for reprocessing
#[update]
async fn reprocess_accept_transfer() -> Result<String, String> {
    if !is_controller(&caller()) {
        return Err("UnAuthorised Access".into());
    }

    let canister_data_ref = CANISTER_DATA.with(|canister_data| { 
        canister_data.borrow().to_owned() });

    if !canister_data_ref.sale_mint_reprocess.is_empty(){
        for (index, key) in canister_data_ref.sale_mint_reprocess.iter().enumerate() {
            let res = mint_approved_nfts(*key);
            match res {
                Ok(_val) => {
                    CANISTER_DATA.with(|canister_data: &RefCell<CanisterData>| {
                        let mut canister_data_ref= canister_data.borrow().to_owned();
                        // let mut col_data = canister_data_ref.collection_data;
                        let _removed_val = canister_data_ref.sale_mint_reprocess.remove(index);
                        *canister_data.borrow_mut() = canister_data_ref;
                    });
                },
                Err(_error_str) => { 
                    continue;
                }
            }
        }
    }

    if !canister_data_ref.sale_transfer_reprocess.is_empty(){
        for (index, key) in canister_data_ref.sale_transfer_reprocess.iter().enumerate() {
            let res = transfer_user_tokens(*key).await;
            match res {
                Ok(_val) => {
                    CANISTER_DATA.with(|canister_data: &RefCell<CanisterData>| {
                        let mut canister_data_ref= canister_data.borrow().to_owned();
                        // let mut col_data = canister_data_ref.collection_data;
                        let _removed_val = canister_data_ref.sale_transfer_reprocess.remove(index);
                        *canister_data.borrow_mut() = canister_data_ref;
                    });
                },
                Err(_error_str) => { 
                    continue;
                }
            }
        }
    }

    Ok("NFTs minted succesfully for all participants".to_string())
}

// call failed mints and transfer for reprocessing
#[update]
async fn reprocess_refund() -> Result<String, String> {
    if !is_controller(&caller()) {
        return Err("UnAuthorised Access".into());
    }

    let sale_refund_reprocess = CANISTER_DATA.with_borrow(|canister_data| { 
        canister_data.sale_refund_reprocess.to_owned() });

    if !sale_refund_reprocess.is_empty(){
        for (index, key) in sale_refund_reprocess.iter().enumerate() {
            let res = refund_user_tokens(*key).await;
            match res {
                Ok(_val) => {
                    CANISTER_DATA.with_borrow_mut(|canister_data| {
                        // let mut canister_data_ref= canister_data.borrow().to_owned();
                        canister_data.sale_refund_reprocess.remove(index);
                    });
                },
                Err(_error_str) => { 
                    continue;
                }
            }
        }
    }

    Ok("NFTs minted succesfully for all participants".to_string())
}


#[update] 
fn get_reprocess_mint() -> Vec<Principal> {

    let reprocess_mint = CANISTER_DATA.with(|canister_data: &RefCell<CanisterData>| { 
        canister_data.borrow().sale_mint_reprocess.to_owned() });

    reprocess_mint
}

#[update] 
fn get_reprocess_transfer() -> Vec<Principal> {

    let reprocess_transfer = CANISTER_DATA.with(|canister_data: &RefCell<CanisterData>| { 
        canister_data.borrow().sale_transfer_reprocess.to_owned() });

    reprocess_transfer
}

#[update] 
fn get_reprocess_refund() -> Vec<Principal> {

    let reprocess_refund = CANISTER_DATA.with(|canister_data: &RefCell<CanisterData>| { 
        canister_data.borrow().sale_refund_reprocess.to_owned() });

    reprocess_refund
}


#[query] 
fn get_total_invested() -> u64 {

    let total_invest = CANISTER_DATA.with(|canister_data: &RefCell<CanisterData>| { 
        canister_data.borrow().total_invested });

    total_invest
}

#[update]
async fn get_user_sale_balance(user_account: Principal) -> Result<(u64, u64), String> {

    let canister_data_ref = CANISTER_DATA.with(|canister_data| { 
        canister_data.borrow().to_owned() });
    let user_balance_map = canister_data_ref.user_balance;
    let user_balance = user_balance_map.get(&user_account).ok_or("No data found for user")?;

    Ok(*user_balance)
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

//pre upgrade
#[ic_cdk::pre_upgrade]
fn pre_upgrade() {
    CANISTER_DATA.with(|canister_data_ref_cell| {
        let canister_data = canister_data_ref_cell.take();

        storage::stable_save((canister_data,)).ok();
    });
}

//post upgrade
#[ic_cdk::post_upgrade]
fn post_upgrade() {
    match storage::stable_restore() {
        Ok((canister_data,)) => {
            CANISTER_DATA.with(|canister_data_ref_cell| {
                *canister_data_ref_cell.borrow_mut() = canister_data;
            });
        }
        Err(_) => {
            panic!("Failed to restore canister data from stable memory");
        }
    }
}

fn allow_only_canister() -> Result<(), String> {
    let canister_id = ic_cdk::api::id();
    if caller() != canister_id {
        Err(String::from("Access denied"))
    } else {
        Ok(())
    }
}

// to add while deploying
fn get_caller() -> Result<Principal, String> {  

    let caller = caller();  
    // The anonymous principal is not allowed to interact with canister.  
    if caller == Principal::anonymous() {  
        Err(String::from("Anonymous principal not allowed to make calls."))  
    } else {  
        Ok(caller)  
    } 

}

ic_cdk::export_candid!();
