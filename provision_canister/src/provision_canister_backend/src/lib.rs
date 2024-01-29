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
use state::{CollectionMetadata, Status, PropertyData};


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

// collection+ NFT metadata
#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct NFT_Metadata {
    pub collection_id: String,
    pub nft_symbol: String,
    pub nft_token_id: String,
    pub nft_uri: String, //image   
    pub collection_name: String,
    pub desc: String,
    pub logo: String, //collection logo 
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

#[update]
async fn create_and_init_canister_with_wasm(id: Principal) -> Result<Principal, String>  {

            println!("New canister created with ID: {:?}", id);

            // let principal_id = new_canister_id.0.canister_id;
            let principal_id = id;

            pub const WASM: &[u8] =
                include_bytes!("//home/shrey/estate-nft/estate_dao_nft/target/wasm32-unknown-unknown/release/estate_dao_nft_backend.wasm.gz");
                // include_bytes!("../../../canister_dummy/target/wasm32-unknown-unknown/release/canister_dummy_backend.wasm");
            
            let wasm_file = WASM.to_vec();

            // create installCodeArgument
            let install_config = InstallCodeArgument {
                mode: CanisterInstallMode::Install,
                wasm_module: wasm_file,
                canister_id: principal_id,
                arg: vec![],
            };
            // Install the Wasm code into the new canister
            let install_result = install_code(install_config).await;

            match install_result {
                Ok(_) => {
                    println!("Wasm code installed successfully!");
                    return Ok(principal_id);
                    // Continue with initialization or other tasks
                }
                Err(err) => {
                    eprintln!("Error installing code: {:?}", err);
                    return Err(err.1);

                }
            }

    // }
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

// calls init collection function of minter to initialize collection metadata
#[update]
async fn init_collection(id: Principal) -> Result<String, String> {
    let res =  call(id, "init_collection", (), ).await; 
        match res{
            Ok(r) => {
                let (res,): (Result<String, String>,) = r;
                res
            },
        Err(_) => Err("Error initializing collection data".to_string())
    }
}


#[update]
async fn mint_NFT(id: Principal, symbol: String, uri: String) -> Result<String, String> {
    
    let canister_id_data = CANISTER_STORE.with(|canister_store| canister_store.borrow().get(&id).cloned());
    if canister_id_data.is_none() {
        return Err("Invalid collection".to_string());
    }
    let asset_can_id = canister_id_data.unwrap().asset_canister;
    let image_uri = "https://".to_string() + &asset_can_id.to_text() + "/" + &uri;

    
    let res =  call(id, "mint", (symbol, image_uri,), ).await; 
        match res{
            Ok(r) => {
                let (res,): (Result<String, String>,) = r;
                res
            },
        Err(_) => Err("Error: minting failed ".to_string())
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
async fn create_and_init_frontend_canister_with_wasm(id: Principal) -> Result<Principal, String>  {

            println!("New canister created with ID: {:?}", id);
            let install_arg = Some(AssetCanisterArgs::InitArgs);

            let serialized_bytes: Vec<u8> = match install_arg {
                // Some(install_args) => serde_json::to_string(&install_arg).unwrap().as_bytes().to_vec(),
                Some(install_args) =>candid::encode_args((install_args,)).expect("Failed to encode arguments"),

                // Some(AssetCanisterArgs::UpgradeArgs) => vec![],
                None => vec![],
            };

            // let principal_id = new_canister_id.0.canister_id;
            let principal_id = id;

            pub const WASM: &[u8] =
                include_bytes!("/home/shrey/work/estate_dao_nft/estate_dao_nft/.dfx/local/canisters/estate_dao_nft_frontend/assetstorage.wasm.gz");
            
            let wasm_file = WASM.to_vec();


            // create installCodeArgument
            let install_config = InstallCodeArgument {
                mode: CanisterInstallMode::Install,
                wasm_module: wasm_file,
                canister_id: principal_id,
                arg: (serialized_bytes),
                // arg: {vec![]},
            };
            // Install the Wasm code into the new canister
            let install_result = install_code(install_config).await;

            match install_result {
                Ok(_) => {
                    println!("Wasm code installed successfully!");
                    return Ok(principal_id);
                    // Continue with initialization or other tasks
                }
                Err(err) => {
                    eprintln!("Error installing code: {:?}", err);
                    return Err(err.1);
                }
            }

    // }
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

    let res =  call(minter_canister, "init_collection", (name, desc, ), ).await; 
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
        asset_canister: asset_canister_id,
        minter_canister
    };

    CANISTER_STORE.with(|canister_store| {
        let mut canister_map =  canister_store.borrow_mut();
        canister_map.insert(minter_canister.clone(), canister_id_data.clone());
    });

    return Ok(canister_id_data);
}

//test  
// #[update]
// fn test_auth_user() -> Result<Vec<Principal>, String> {

//     let caller = caller();
//     let user = Principal::from_text("e4j7x-faktm-kmxvh-lsmry-esxyc-roihr-ycta2-6rv22-kxxyd-jugcj-tae").unwrap(); 
//     let mut minter_canister_vec: Vec<Principal> = Vec::new();
//     minter_canister_vec.push(caller);
//     minter_canister_vec.push(user);
   
   
//     return Ok(minter_canister_vec);  

// }

#[update]
fn get_all_minter_canisters() -> Result<Vec<Principal>, String> {

    let user = Principal::from_text("e4j7x-faktm-kmxvh-lsmry-esxyc-roihr-ycta2-6rv22-kxxyd-jugcj-tae").unwrap(); 
    if caller() != Principal::self_authenticating(user) {
        return Err("unathorized user".to_string());
    }
    else{

        CANISTER_STORE.with(|canister_store| {
            let canister_map = canister_store.borrow_mut();
            if canister_map.to_owned().is_empty() {
                return Err("Empty Canister List".to_string());
            }
            let mut minter_canister_vec: Vec<Principal> = Vec::new();
            for (_key, value) in canister_map.to_owned().iter() {
                minter_canister_vec.push(value.to_owned().minter_canister);
            }
            return Ok(minter_canister_vec);  
        })
    }
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


// #[query]
// async fn get_canister_list_data(id: Principal) -> Result<Vec<String>, String> {
//     let canister_list = CANISTER_STORE.with(|canister_store| {
//         canister_store.borrow().to_owned();
//     });
//         if canister_list.is_empty() {
//             return Err("Empty Canister List".to_string());
//         }
//     let res =  call(id, "get_prop_data", (), ).await; 
//         match res{
//             Ok(r) => {
//                 let (res,): (Result<Vec<String>, String>,) = r;
//                 res
//             },
//         Err(_) => Err("Error displaying collection images".to_string())
//     }
// }

#[update]
async fn filter_status() -> Result<Vec<Principal>, String> {

    let collection_list = get_all_minter_canisters().unwrap();
    let mut filtered_list:Vec<Principal> = Vec::new();

    for col in collection_list{
        let result =  call(col, "get_collection_status", (), ).await; 

        match result{
            Ok(r) => {
                let (res,): (Result<Status, String>,) = r;
                match res {
                    Ok(s) => match s{
                        Status::Upcoming=>{filtered_list.push(col)}
                        Status::Live => todo!(),
                        Status::Ended => todo!(),}
                    Err(_) => return Err("Error fetching collection status".to_string())
                }
            },
        Err(_) => return Err("Error fetching collection data call".to_string())
        }
    }

        // let coll_data_result =  call(col, "get_collection_metadata", ()).await; 
        // match coll_data_result{
        //     Ok(r) => {
        //         let (res,): (Result<CollectionMetadata, String>,) = r;
        //         // let coll_data: CollectionMetadata = coll_data_result.unwrap();
        //         match res {
        //             Ok(_) => match res.unwrap().status{
        //                 Status::Upcoming=>{filtered_list.push(col)}
        //                 Status::Live => todo!(),
        //                 Status::Ended => todo!(),}
        //             Err(_) => println!("dd")
        //         }
        //     },
        //     Err(_) => println!("dd")
        // };

        // let coll_data: CollectionMetadata = coll_data_result.unwrap();
        // match coll_data.status{
        //     Status::Upcoming => {filtered_list.push(col)},
        //     _ => println!("")
        // }
    // }
    //todo loop

        // let mut add_meta = col_data.additional_metadata.unwrap();
        
        // add_meta.documents = doc_details;
        // col_data.additional_metadata = Some(add_meta);

        // *co in l_data.borrow_mut() = col_data;

        return Ok(filtered_list);
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
 
// Enable Candid export
ic_cdk::export_candid!();
