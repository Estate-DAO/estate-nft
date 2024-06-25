mod state;

use ic_cdk::api::{self, is_controller};
use ic_cdk::api::management_canister::main::{create_canister, install_code, CreateCanisterArgument, CanisterInstallMode, InstallCodeArgument, CanisterSettings};
use ic_cdk::api::call::{call, call_with_payment, CallResult,RejectionCode };
use candid::{CandidType, Principal, Deserialize};
use ic_cdk::api::management_canister::provisional::CanisterIdRecord;
use ic_cdk::{caller, notify, query, storage, update};
use serde::Serialize;
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::vec;
// use argon2::{Argon2, PasswordHash, PasswordVerifier, Variant, Version};

use state::{AdditionalMetadata, ApprovedResponse, CanisterIds, FinancialDetails, FormMetadata, MarketDetails, Metadata, PropertyDetails, SaleData, SaleStatus, Status};


#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct SetPermissions{
    pub prepare: Vec<Principal>,
    pub commit: Vec<Principal>,
    pub manage_permissions: Vec<Principal>
}

#[derive(CandidType)]
struct EmptyArgs {}

type FormData = BTreeMap<u16, FormMetadata>;
type CanisterStore = BTreeMap<Principal, CanisterIds>;

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct Config{
    pub authorised_principal: Principal,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            authorised_principal: Principal::anonymous(), 
        }
    }
}

#[derive(Clone, Debug, CandidType, Default, Deserialize, Serialize)]
pub struct CanisterData { 
    pub form_data: FormData,
    pub form_counter: u16,
    pub wasm_store: WasmStore,
    pub canister_store: CanisterStore,
    pub stored_key: String,
    pub known_principals: Vec<Principal>,
    pub config: Config,
}

thread_local! {
    static CANISTER_DATA: RefCell<CanisterData> = RefCell::default();
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct InitArgs;

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct UpgradeArgs{
    pub set_permissions: Option<SetPermissions>
}

#[derive(Clone, Debug, Default, CandidType, Deserialize, Serialize)]
pub struct WasmStore{
    pub minter_wasm_blob: Vec<u8>,
    pub asset_wasm_blob: Vec<u8>,
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub enum AssetCanisterArgs{
    InitArgs,
    UpgradeArgs
}

#[derive(thiserror::Error, Debug)]  
enum ProvisionError {
    #[error("minter canister not initalized")]
    MinterCanisterNotInitialized(),
    #[error("Invalid data: {0}")]
    InvalidData(String),
}

#[update] 
fn add_known_principals( 
    new_admin: Principal,
) -> Result<String, String> {

    if !is_controller(&caller()) {
        return Err("UnAuthorised Access".into());
    }

    CANISTER_DATA.with_borrow_mut(|canister_data| { 
        // let principal_list = canister_data.known_principals });

        if canister_data.known_principals.contains(&new_admin) {
            return Err("admin already added".to_string());
        } 
        canister_data.known_principals.push(new_admin);

        Ok("new admin added succesfully".to_string())
    })
}

#[query] 
fn verify_admin( 
    caller: Principal,
) -> bool {

    let principal_list = CANISTER_DATA.with(|canister_data| { 
        canister_data.borrow().known_principals.to_owned() });

    if principal_list.contains(&caller) {
        return true;
    } 
    false
}

#[query] 
fn get_admins() -> Vec<Principal> {

    let principal_list = CANISTER_DATA.with(|canister_data| { 
        canister_data.borrow().known_principals.to_owned() });

    principal_list
}

#[update] 
fn remove_known_principals( 
    admin_to_remove: Principal,
) -> Result<Vec<Principal>, String> {

    if !is_controller(&caller()) {
        return Err("UnAuthorised Access".into());
    }

    CANISTER_DATA.with_borrow_mut(|canister_data| { 
        // let principal_list = canister_data.known_principals });

        if !canister_data.known_principals.contains(&admin_to_remove) {
            return Err("admin does not exist".to_string());
        } 
        canister_data.known_principals.retain(|&x| x != admin_to_remove);

        Ok(canister_data.known_principals.to_owned())
    })
}

#[update]
fn set_authorised_principal(new_principal: Principal) -> Result<String, String> {
    let caller = caller();
    if caller != Principal::from_text("v3mpp-bismc-wjug7-6t6jc-iqu2b-64xh3-rpwld-sw5e2-fsecm-6lfss-aqe").unwrap() {
        return Err("UnAuthorised Access".into());
    }
    CANISTER_DATA.with_borrow_mut(|canister_data| { 
        canister_data.config.authorised_principal = new_principal;

        Ok("authorised principal added succesfully".to_string())
    })
}

#[query]
fn get_authorised_principal() -> Result<Principal, String> {
    let caller = caller();
    if caller != Principal::from_text("v3mpp-bismc-wjug7-6t6jc-iqu2b-64xh3-rpwld-sw5e2-fsecm-6lfss-aqe").unwrap()
    {    
        return Err("UnAuthorised Access".into());
    }
    let authorised_principal = CANISTER_DATA.with(|canister_data| { 
        canister_data.borrow().config.authorised_principal.to_owned() });
    
    return Ok(authorised_principal);

}


#[update] 
fn update_key( 
    new_str: String,
) -> Result<String, String> {

    if !is_controller(&caller()) {
        return Err("UnAuthorised Access".into());
    }

    CANISTER_DATA.with_borrow_mut(|canister_data| {   
        canister_data.stored_key = new_str;       
        Ok("key updated succesfully".to_string())
    })

}

#[query] 
fn verify_key( 
    key: String,
) -> bool {

    CANISTER_DATA.with_borrow(|canister_data| {

        if canister_data.stored_key == key{
            return true;
        }
        false
    })
}

#[update] 
fn init_minter_wasm( 
    wasm: Vec<u8>,
) -> Result<String, String> {
    if !is_controller(&caller()) {
        return Err("UnAuthorised Access".into());
    }

    CANISTER_DATA.with_borrow_mut(|canister_data| {
        canister_data.wasm_store.minter_wasm_blob = wasm;

        Ok("minter set succesfully".to_string())
    })
}

#[update] 
fn init_asset_wasm( 
    wasm: Vec<u8>,
) -> Result<String, String> {
    if !is_controller(&caller()) {
        return Err("UnAuthorised Access".into());
    }

    CANISTER_DATA.with_borrow_mut(|canister_data| {
        canister_data.wasm_store.asset_wasm_blob = wasm;

        Ok("asset wasm set succesfully".to_string())
    })
}

//collection specific data
#[update] 
fn get_form_metadata( 
    index: u16 
) -> Result<FormMetadata, String> {

    CANISTER_DATA.with_borrow(|canister_data| {
        let form_data = canister_data.form_data.get(&index).cloned().ok_or("no data for this index".to_string())?;
        Ok(form_data)
    })
}


#[update]
fn grant_commit_permission(id: Principal, user_id: Principal) -> Result<String, String> {
    let res = notify(id, "authorize", (user_id,), ); 
  
    match res{
        Ok(r) => Ok("success".to_string()),
        Err(_) => Err("Failed to authorize".to_string())
    }
}

#[update]
fn revoke_commit_permission(id: Principal, user_id: Principal) -> Result<String, String> {
    let res = notify(id, "deauthorize", (user_id,), ); 

    match res{
        Ok(r) => Ok("success".to_string()),
        Err(_) => Err("Failed to authorize".to_string())
    }
}


#[update]
fn get_all_minter_canisters() -> Result<Vec<Principal>, String> {

    CANISTER_DATA.with_borrow_mut(|canister_data| {
        let mut minter_canister_vec: Vec<Principal> = Vec::new();

        if canister_data.canister_store.is_empty() {
            return Ok(minter_canister_vec);
        }
        for (_key, value) in canister_data.canister_store.iter() {
            minter_canister_vec.push(value.to_owned().minter_canister);
        }
        Ok(minter_canister_vec)
    })
}

#[query]
fn get_all_canisters() -> Result<Vec<CanisterIds>, String> {

    CANISTER_DATA.with_borrow_mut(|canister_data| {
        let mut canister_vec: Vec<CanisterIds> = Vec::new();

        if canister_data.canister_store.is_empty() {
            return Ok(canister_vec);
        }
        for (_key, value) in canister_data.canister_store.iter() {
            canister_vec.push(value.to_owned());
        }
        Ok(canister_vec)
    })
}

#[update]
async fn filter_status(stat: Status) -> Result<Vec<Principal>, String> {

    let collection_list = get_all_canisters();
    match collection_list {
        Ok(col_list) => {
            let mut filtered_list:Vec<Principal> = Vec::new();

            for col in col_list{
                let result =  call(col.minter_canister, "get_collection_status", (), ).await; 

                match result{
                    Ok(r) => {
                        let (res,): (Result<Status, String>,) = r;
                        match res {
                            Ok(s) => {
                                if s == stat{
                                    filtered_list.push(col.minter_canister);
                                }
                                else{
                                    continue;
                                }
                            }
                            Err(_) =>{ return Err("Error fetching collection status".to_string())}
                        }
                    },
                Err(_) => return Err("Error fetching collection data call".to_string())
                }
            }
            return Ok(filtered_list);
        }
        Err(e) => return Err(e),
    }

}


#[update]
async fn approve_collection(index: u16, approval: bool) -> Result<ApprovedResponse, String> {

    let caller = caller();
    if !verify_admin(caller) {
        return Err("unauthorised user".to_string());
    }
    
    let form_list = CANISTER_DATA.with_borrow(|canister_data|{canister_data.form_data.to_owned()});

    let form_data = form_list.get(&index)
        .ok_or("no form data for the index")?;

    if approval {

        let wasms = CANISTER_DATA.with_borrow(|canister_data|{canister_data.wasm_store.to_owned()});

        let settings = CanisterSettings {
            controllers: Some(vec![ api::id()]),
            compute_allocation: None,
            memory_allocation: None,
            freezing_threshold: None,
            reserved_cycles_limit: None
        };
        let create_arg = CreateCanisterArgument{
            settings: Some(settings),
        };

        let (canister_id_1,): (CanisterIdRecord,) = 
            create_canister(
                create_arg.clone(),
                200_000_000_000,
            )
            .await.expect("Failed to create canister");

        let install_arg = Some(AssetCanisterArgs::InitArgs);
        // let arg_vec = install_arg.as_bytes()

        let serialized_bytes: Vec<u8> = match install_arg {
            // Some(install_args) => serde_json::to_string(&install_arg).unwrap().as_bytes().to_vec(),
            Some(install_args) =>candid::encode_args((install_args,)).expect("Failed to encode arguments"),

            None => vec![],
        };

        // let principal_id = new_canister_id.0.canister_id;
        let asset_canister_id = canister_id_1.canister_id;

        let asset_wasm = wasms.asset_wasm_blob;

        // create installCodeArgument
        let install_config = InstallCodeArgument {
            mode: CanisterInstallMode::Install,
            wasm_module: asset_wasm,
            canister_id: asset_canister_id,
            arg: (serialized_bytes),
            // arg: {vec![]},

        };
        // Install the Wasm code into the new canister
        let install_result = install_code(install_config).await;

        match install_result {
            Ok(_) => {}
            Err(err) => {
                eprintln!("Error installing code: {:?}", err);
                return Err(err.1);
            }
        }

        // create minter canister 
        let (canister_id_2,): (CanisterIdRecord,) = 
        create_canister(
            create_arg.clone(),
            200_000_000_000,
        )
        .await.expect("Failed to create canister");

        // install minter canister
        let minter_canister_id = canister_id_2.canister_id;

        let minter_wasm = wasms.minter_wasm_blob;
        // let arg_vec: Vec<u8> = Vec::new();
        let encoded_args = candid::encode_args(()).expect("Failed to encode arguments");
        // let empty_args = EmptyArgs {};
        // let encoded_args = candid::encode_args((empty_args,)).expect("Failed to encode arguments");

        // create installCodeArgument
        let install_config_minter = InstallCodeArgument {
            mode: CanisterInstallMode::Install,
            wasm_module: minter_wasm,
            canister_id: minter_canister_id,
            arg: encoded_args,
            // arg: {vec![]},
            // arg: candid::encode_one(()).unwrap(),
        };

        // Install the Wasm code into the new canister
        let install_result = install_code(install_config_minter).await;

        match install_result {
            Ok(_) => {}
            Err(err) => {
                eprintln!("Error installing code: {:?}", err);
                return Err(err.1);
            }
        }


        let res = call(minter_canister_id, "init_collection", (form_data,), ).await; 
        
        match res{
            Ok(r) => {
                let (res,): (Result<String, String>,) = r;
            }, 
            Err(_) =>{            
                return Err("error initializing collection".to_string());
            }
        };
        
        CANISTER_DATA.with_borrow_mut(|canister_data| {
            // let mut canister_data_ref = canister_data.borrow_mut().to_owned();

            canister_data.form_data.remove(&index);

            let canister_id_data = CanisterIds{
                asset_canister: asset_canister_id,
                minter_canister: minter_canister_id
            };

            canister_data.canister_store.insert(minter_canister_id.clone(), canister_id_data.clone());

            Ok(ApprovedResponse::CanisterId(canister_id_data))
        })
    }
    else {
         CANISTER_DATA.with_borrow_mut(|canister_data| {

            canister_data.form_data.remove(&index);

            Ok(ApprovedResponse::StrResp("collection rejected".to_string()))
        })
    }
}

#[update] 
fn init_form_metadata( 
    form_input: FormMetadata
) -> Result<String, String> {
    //admin verification not requiered, anyone can fill form
    // let caller = caller();
    // if !verify_admin(caller) {
    //     return Err("unauthorised user".to_string());
    // }

    CANISTER_DATA.with_borrow_mut(|canister_data| {
        let mut form_data = form_input;
        form_data.status = Status::Draft;

        canister_data.form_counter.saturating_add(1);
    
        canister_data.form_data.insert(canister_data.form_counter, form_data);
    
        Ok("form initiated succesfully".to_string())
    })
}

//collection specific data
#[update] 
fn get_form_list( 
) -> BTreeMap<u16, FormMetadata> {

    CANISTER_DATA.with(|canister_data| {
        let form_list =  canister_data.borrow().form_data.to_owned();
        
        form_list
    })
}

//todo
fn check_unique_name(name: String) -> bool {
    let mut minter_canister_vec = Vec::new();
    match get_all_minter_canisters() {
        Ok(vec) => {
            minter_canister_vec = vec;
            for col in minter_canister_vec {
                //call get_name function of minter
                let name_str = String::from("nnnn");
                if name == name_str{
                    return false;
                }
            }
            true
        }
        Err(_) => {
            false        
        }
    }
}

// todo
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

#[update]
async fn get_escrow_balance(minter: Principal, user_id: Principal) -> Result<u64, String> {  

    let res =  call(minter, "get_balance", (user_id,), ).await; 
    match res{
        Ok(r) => {
            let (res,): (Result<u64, String>,) = r;
            res
        }, 
        Err(e) =>{
            Err(e.1)
        }
    }
}

#[update]
async fn get_sale_balance(minter: Principal, user_id: Principal) -> Result<(u64, u64), String> {  

    let res =  call(minter, "get_user_sale_balance", (user_id,), ).await; 
    match res{
        Ok(r) => {
            let (res,): (Result<(u64, u64), String>,) = r;
            res
        }, 
        Err(e) =>{
            Err(e.1)
        }
    }
}

// #[update]
// async fn sale_confirmed_mint(minter: Principal) -> Result<String, String> {  

//     if !is_controller(&caller()) {
//         return Err("UnAuthorised Access".into());
//     }

//     let res =  call(minter, "sale_confirmed_mint", (), ).await; 
//     match res{
//         Ok(r) => {
//             let (res,): (Result<String, String>,) = r;
//             res
//         }, 
//         Err(_) =>{
//             Err("error".to_string())
//         }
//     }
// }

#[update]
async fn sale_accept(minter: Principal) -> Result<String, String> {  

    if !is_controller(&caller()) {
        return Err("UnAuthorised Access".into());
    }

    let res =  call(minter, "sale_accepted", (), ).await; 
    match res{
        Ok(r) => {
            let (res,): (Result<String, String>,) = r;
            res
        }, 
        Err(e) =>{
            Err(e.1)
        }
    }
}


#[update]
async fn sale_confirmed_refund(minter: Principal) -> Result<String, String> { 

    if !is_controller(&caller()) {
        return Err("UnAuthorised Access".into());
    } 

    let res: Result<(Result<String, String>,), (RejectionCode, String)> =  call(minter, "sale_rejected", (), ).await; 
    match res{
        Ok(r) => {
            let (res,): (Result<String, String>,) = r;
            res
        },
        Err(e) =>{
            Err(e.1)
        }
    }
}

// #[update]
// async fn sale_confirmed_accept_payment(minter: Principal) -> Result<String, String> { 

//     if !is_controller(&caller()) {
//         return Err("UnAuthorised Access".into());
//     } 

//     let res =  call(minter, "sale_confirmed_transfer", (), ).await; 
//     match res{
//         Ok(r) => {
//             let (res,): (Result<String, String>,) = r;
//             res
//         }, 
//         Err(_) =>{
//             Err("error".to_string())
//         }
//     }
// }

#[update]
async fn get_nft_metadata(minter: Principal, token_id: String) -> Result<Metadata, String> {  

    let res =  call(minter, "get_metadata", (token_id,), ).await; 
    match res{
        Ok(r) => {
            let (res,): (Result<Metadata, String>,) = r;
            res
        }, 
        Err(e) =>{
            Err(e.1)
        }
    }
}

#[update]
async fn get_sale_data(minter: Principal, token_id: String) -> Result<SaleData, String> {  

    let res =  call(minter, "get_sale_data", (token_id,), ).await; 
    match res{
        Ok(r) => {
            let (res,): (Result<SaleData, String>,) = r;
            res
        }, 
        Err(e) =>{
            Err(e.1)
        }
    }
}

#[update]
async fn get_total_invested(minter: Principal) -> Result<u64, String> {  

    let res =  call(minter, "get_total_invested", (), ).await; 
    match res{
        Ok(r) => {
            let (res,): (u64,) = r;
            Ok(res)
        }, 
        Err(e) =>{
            Err(e.1)
        }
    }
}

//for reprocessing failed accept payment and mint
#[update]
async fn reprocess_sale_accept(minter: Principal) -> Result<String, String> {  

    if !is_controller(&caller()) {
        return Err("UnAuthorised Access".into());
    }

    let res =  call(minter, "reprocess_accept_transfer", (), ).await; 
    match res{
        Ok(r) => {
            let (res,): (Result<String, String>,) = r;
            res
        }, 
        Err(e) =>{
            Err(e.1)
        }
    }
}

//for reprocessing failed accept payment and mint
#[update]
async fn reprocess_sale_refund(minter: Principal) -> Result<String, String> {  

    if !is_controller(&caller()) {
        return Err("UnAuthorised Access".into());
    }

    let res =  call(minter, "reprocess_refund", (), ).await; 
    match res{
        Ok(r) => {
            let (res,): (Result<String, String>,) = r;
            res
        }, 
        Err(e) =>{
            Err(e.1)
        }
    }
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
                // canister_data_ref_cell.borrow_mut().known_principal_ids.insert(KnownPrincipalType::CanisterIdSnsGovernance, Principal::from_str(GOVERNANCE_CANISTER_ID).unwrap());
            });
        }
        Err(_) => {
            panic!("Failed to restore canister data from stable memory");
        }
    }
}

// Enable Candid export
ic_cdk::export_candid!();
