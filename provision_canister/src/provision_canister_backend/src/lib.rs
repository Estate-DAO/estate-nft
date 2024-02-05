mod state;

use candid::types::principal;
use ic_cdk::api::management_canister::main::{create_canister, install_code, CreateCanisterArgument, CanisterInstallMode, InstallCodeArgument, CanisterSettings};
use ic_cdk::api::call::{call, call_with_payment, CallResult,RejectionCode };
use candid::{CandidType, Principal, Deserialize};
use ic_cdk::api::management_canister::provisional::CanisterIdRecord;
use ic_cdk::{caller, notify, query, update};
use std::fs::{read, Metadata};
use serde::Serialize;
use std::cell::RefCell;
use std::collections::BTreeMap;
use state::{CollectionMetadata, PropDetails, PropertyData, Status};


#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct SetPermissions{
    pub prepare: Vec<Principal>,
    pub commit: Vec<Principal>,
    pub manage_permissions: Vec<Principal>
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct CanisterIds{
    pub asset_canister: Principal,
    pub minter_canister: Principal,
}


#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct InitArgs;

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct UpgradeArgs{
    pub set_permissions: Option<SetPermissions>
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub enum AssetCanisterArgs{
    InitArgs,
    UpgradeArgs
}

type CanisterStore = BTreeMap<Principal, CanisterIds>;

thread_local! {
    static CANISTER_STORE: RefCell<CanisterStore> = RefCell::default();
}

#[derive(thiserror::Error, Debug)]  
enum ProvisionError {
    #[error("minter canister not initalized")]
    MinterCanisterNotInitialized(),
    #[error("Invalid data: {0}")]
    InvalidData(String),
}

// collection+ NFT metadata
#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct NFT_Metadata {
    pub collection_id: String,
    pub nft_symbol: String,
    pub nft_token_id: String,
    pub nft_uri: String, //image   
    pub collection_name: String,
    pub desc: String,
    pub royalty_percent: u16,
    pub total_supply: u16,
    pub supply_cap: u16,
}

#[update]
async fn create_new_canister() -> Result<Principal, String> {
    let settings = CanisterSettings::default();
    let create_arg = CreateCanisterArgument{
        settings: Some(settings)
    };

    let (canister_id,): (CanisterIdRecord,) = match call_with_payment(
        Principal::management_canister(), // Management canister address
        "create_canister", // Function name
        (create_arg,), // Argument tuple
        5_000_000_000_000, // Payment amount in cycles
    ).await {
        Ok(x) => x,
        Err((_, _)) => (CanisterIdRecord { canister_id: Principal::anonymous() },),
    };

    if canister_id.canister_id == Principal::anonymous() {
        return Err("error creating canister".to_string());
    } else {
        return Ok(canister_id.canister_id);
    }
    
}


#[query(composite = true)]
async fn call_fun(id: Principal) -> String {
    match call(id, "check_call", (), ).await {
        Ok(r) => {
            let (res,): (String,) = r;
            res
        },
        Err(_) => "dummy".to_string()
    }
}

#[update]
async fn get_token_metadata(id: Principal, token_id: String) -> Result<NFT_Metadata, String> {
    let res =  call(id, "get_metadata", (token_id,), ).await; 
        match res{
            Ok(r) => {
                let (res,): (Result<NFT_Metadata, String>,) = r;
                res
            },
        Err(_) => Err("Error displaying metedata".to_string())
    }
}


#[update]
fn grant_commit_permission(id: Principal, user_id: Principal) -> String {
    // ic_cdk::println!("Get in .. using backend={}", id.to_text());
    let res = notify(id, "authorize", (user_id,), ); 
  
    match res{
        Ok(()) => {"success".to_string()}
        Err(_) => {"error".to_string()}
    }
}

#[update]
fn revoke_commit_permission(id: Principal, user_id: Principal) -> String {
    // ic_cdk::println!("Get in .. using backend={}", id.to_text());
    let res = notify(id, "deauthorize", (user_id,), ); 

    match res{
        Ok(()) => {"success".to_string()}
        Err(_) => {"error".to_string()}
    }
}

#[update]
async fn all_canister_create(name: String, desc: String) -> Result<CanisterIds, String> {
    let settings = CanisterSettings::default();
    let create_arg = CreateCanisterArgument{
        settings: Some(settings)
    };

    let (canister_id_1,): (CanisterIdRecord,) = match call_with_payment(
        Principal::management_canister(), // Management canister address
        "create_canister", // Function name
        (create_arg,), // Argument tuple
        7_000_000_000_000, // Payment amount in cycles
    ).await {
        Ok(x) => x,
        Err((_, _)) => (CanisterIdRecord { canister_id: Principal::anonymous() },),
    };

    if canister_id_1.canister_id == Principal::anonymous() {
        return Err("error creating asset canister".to_string());
    } 

    let install_arg = Some(AssetCanisterArgs::InitArgs);
    // let arg_vec = install_arg.as_bytes()

    let serialized_bytes: Vec<u8> = match install_arg {
        // Some(install_args) => serde_json::to_string(&install_arg).unwrap().as_bytes().to_vec(),
        Some(install_args) =>candid::encode_args((install_args,)).expect("Failed to encode arguments"),

        None => vec![],
    };

    // let principal_id = new_canister_id.0.canister_id;
    let asset_canister_id = canister_id_1.canister_id;

    pub const WASM: &[u8] =
        include_bytes!("/home/shrey/work/new_asset/.dfx/local/canisters/new_asset_frontend/assetstorage.wasm.gz");
    
    let wasm_file = WASM.to_vec();

    // create installCodeArgument
    let install_config = InstallCodeArgument {
        mode: CanisterInstallMode::Install,
        wasm_module: wasm_file,
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
    let settings = CanisterSettings::default();
    let create_arg = CreateCanisterArgument{
        settings: Some(settings)
    };

    let (canister_id_2,): (CanisterIdRecord,) = match call_with_payment(
        Principal::management_canister(), // Management canister address
        "create_canister", // Function name
        (create_arg,), // Argument tuple
        4_000_000_000_000, // Payment amount in cycles
    ).await {
        Ok(x) => x,
        Err((_, _)) => (CanisterIdRecord { canister_id: Principal::anonymous() },),
    };

    if canister_id_2.canister_id == Principal::anonymous() {
        return Err("error creating asset canister".to_string());
    } 

    // install minter canister
    let minter_canister = canister_id_2.canister_id;

    pub const MINTERWASM: &[u8] =
        include_bytes!("/home/shrey/estate-nft/estate_dao_nft/target/wasm32-unknown-unknown/release/estate_dao_nft_backend.wasm.gz");
        // include_bytes!("../../../canister_dummy/target/wasm32-unknown-unknown/release/canister_dummy_backend.wasm");

    let wasm_file = MINTERWASM.to_vec();

    // create installCodeArgument
    let install_config = InstallCodeArgument {
        mode: CanisterInstallMode::Install,
        wasm_module: wasm_file,
        canister_id: minter_canister,
        arg: vec![],
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

    //remove
    let mut e:String = String::from("");

    //todo add caller
    // let user = caller();

    let user = Principal::from_text("e4j7x-faktm-kmxvh-lsmry-esxyc-roihr-ycta2-6rv22-kxxyd-jugcj-tae").unwrap(); 


    let res =  call(minter_canister, "init_collection", (name, desc, user.clone()), ).await; 
        match res{
            Ok(r) => {
                let (res,): (Result<String, String>,) = r;
            }, 
        Err(_) =>{e=String::from("error")}
    }

    if e == "error".to_string(){
        return Err("error initializing struct".to_string());
    }

    let canister_id_data = CanisterIds{
        asset_canister: user,
        minter_canister
        // minter_canister: user
    };

    CANISTER_STORE.with(|canister_store| {
        let mut canister_map =  canister_store.borrow_mut();
        canister_map.insert(minter_canister.clone(), canister_id_data.clone());
    });

    return Ok(canister_id_data);
}

//test  
#[update]
async fn test_auth_user() -> Result<Vec<Principal>, String> {

    let caller = caller();
    let mut minter_canister_vec: Vec<Principal> = Vec::new();
    minter_canister_vec.push(caller);

    // let res =  call(id, "get_collection_metadata", (), ).await; 
    // match res{
    //         Ok(r) => {
    //             let (res,): (Result<CollectionMetadata, String>,) = r;
    //             let prop_owner = res.unwrap().owner;
    //             let prop_owner_principal = Principal::from_text(prop_owner).unwrap();
                
    //             minter_canister_vec.push(prop_owner_principal);
    //         },
    //     Err(e) => {}
    // }

    return Ok(minter_canister_vec);  
}

#[update]
fn get_all_minter_canisters() -> Result<Vec<Principal>, String> {

    CANISTER_STORE.with(|canister_store| {
        let mut minter_canister_vec: Vec<Principal> = Vec::new();

        let canister_map = canister_store.borrow_mut();
        if canister_map.to_owned().is_empty() {
            return Ok(minter_canister_vec);
        }
        for (_key, value) in canister_map.to_owned().iter() {
            minter_canister_vec.push(value.to_owned().minter_canister);
        }
        return Ok(minter_canister_vec);  
    })
}


// #[update]
// async fn get_all_canisters() -> Result<Vec<(Principal, PropertyData)>, String> {

//     CANISTER_STORE.with(|canister_store| {
//         let canister_map = canister_store.borrow();
//         if canister_map.to_owned().is_empty() {
//             return Err("Empty Canister List".to_string());
//         }

//         // let mut canister_vec: Vec<CanisterIds> = Vec::new();
//         // for (_key, value) in canister_map.to_owned().iter() {
//         //     canister_vec.push(value.to_owned());
//         // }

//         let mut coll_vec: Vec<(Principal, PropertyData)> = Vec::new();

//         //try 
//         // let tasks: Vec<_> = 

//         //     .map(|item| tokio::spawn(item.resolve()))
//         //     .collect();
//         // // now await them to get the resolve's to complete
//         // for task in tasks {
//         //     task.await.unwrap();
//         // }

//         let mut vec1 = Vec::new();

//         for (_key, value) in canister_map.to_owned().iter() {
//             let result = call(value.to_owned().minter_canister, "get_prop_data", (), ); 
//             vec1.push(result);
//         }

//         vec1.into_iter().map(|item| tokio::spawn(item.await))
//             .collect();

//         for (_key, value) in canister_map.to_owned().iter() {

//             match result{
//                 Ok(r) => {
//                     let (res,): (Result<PropertyData, String>,) = r;
//                     match res{
//                         Ok(data) => {
//                             coll_vec.push((value.to_owned().minter_canister, data));
//                         },
//                         Err(_) => {return Err("error fetching struct".to_string());}
//                     }
//                 }
//                 Err(_) => {return Err("error fetching".to_string());}
//             }
//         }
//         return Ok(coll_vec);  
//     })
// }


#[update]
async fn filter_status(stat: Status) -> Result<Vec<Principal>, String> {

    let collection_list = get_all_minter_canisters();
    match collection_list {
        Ok(col_list) => {
            let mut filtered_list:Vec<Principal> = Vec::new();

            for col in col_list{
                let result =  call(col, "get_collection_status", (), ).await; 
        
                match result{
                    Ok(r) => {
                        let (res,): (Result<Status, String>,) = r;
                        match res {
                            Ok(s) => {
                                if s == stat{
                                    filtered_list.push(col);
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

#[query]
async fn get_collection_images(id: Principal) -> Result<Vec<String>, String> {
    let res =  call(id, "collection_image", (), ).await; 
        match res{
            Ok(r) => {
                let (res,): (Result<Vec<String>, String>,) = r;
                res
            },
        Err(_) => Err("Error displaying collection images".to_string())
    }
}
 

// calls init collection function of minter to initialize collection metadata
#[update]
async fn call_update_prop(id: Principal, prop_det: PropDetails) -> Result<String, String> {
    let res =  call(id, "update_prop_det", (prop_det,), ).await; 
        match res{
            Ok(r) => {
                let (res,): (Result<String, String>,) = r;
                res
            },
        Err(e) => Err(e.1)
    }
    
}

// #[update]
// async fn call_get_metadata(id: Principal) -> Result<CollectionMetadata, String> {
//     let res =  call(id, "get_collection_metadata", (), ).await; 
//         match res{
//             Ok(r) => {
//                 let (res,): (Result<CollectionMetadata, String>,) = r;
//                 res
//             },
//         Err(e) => Err(e.1)
//     }
// }

// Enable Candid export
ic_cdk::export_candid!();
