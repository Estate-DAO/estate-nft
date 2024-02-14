mod state;

use ic_cdk::api::is_controller;
use ic_cdk::api::management_canister::main::{create_canister, install_code, CreateCanisterArgument, CanisterInstallMode, InstallCodeArgument, CanisterSettings};
use ic_cdk::api::call::{call, call_with_payment, CallResult,RejectionCode };
use candid::{CandidType, Principal, Deserialize};
use ic_cdk::api::management_canister::provisional::CanisterIdRecord;
use ic_cdk::{caller, notify, query, update};
use serde::Serialize;
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};

use state::{AdditionalMetadata, FormMetadata, FinancialDetails, MarketDetails, PropertyDetails, SaleData, SaleStatus, Status, CanisterIds};


#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct SetPermissions{
    pub prepare: Vec<Principal>,
    pub commit: Vec<Principal>,
    pub manage_permissions: Vec<Principal>
}

type FormData = BTreeMap<u16, FormMetadata>;
// type FormDataPropDetails = BTreeMap<u16, PropertyDetails>;

thread_local! {
    static ADMIN_ACCOUNT: RefCell<String> = RefCell::default();
    static FORM_DATA: RefCell<FormData> = RefCell::default();
    static COUNTER: RefCell<u16> = RefCell::default();
    static WASM_STORE: RefCell<WasmStore> = RefCell::default();
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

#[update] 
fn init_minter_wasm( 
    wasm: Vec<u8>,
) -> Result<String, String> {

    WASM_STORE.with(|wasms| {
        wasms.borrow_mut().minter_wasm_blob = wasm;
        // let mut wasms =  form_list.borrow_mut();
        // form_list.insert(key, form_input);
    });

    Ok("minter set succesfully".to_string())
}

#[query] 
fn get_minter_wasm( 
) -> Result<Vec<u8>, String> {

    WASM_STORE.with(|wasms| {
        let wasm = wasms.borrow().to_owned();
        Ok(wasm.minter_wasm_blob)

        // let mut wasms =  form_list.borrow_mut();
        // form_list.insert(key, form_input);
    })
}

#[update] 
fn init_asset_wasm( 
    wasm: Vec<u8>,
) -> Result<String, String> {

    WASM_STORE.with(|wasms| {
        wasms.borrow_mut().asset_wasm_blob = wasm;
        // let mut wasms =  form_list.borrow_mut();
        // form_list.insert(key, form_input);
    });

    Ok("asset wasm set succesfully".to_string())
}


#[query] 
fn get_asset_wasm( 
) -> Result<Vec<u8>, String> {

    WASM_STORE.with(|wasms| {
        let wasm = wasms.borrow().to_owned();
        Ok(wasm.asset_wasm_blob)

        // let mut wasms =  form_list.borrow_mut();
        // form_list.insert(key, form_input);
    })
}

//collection specific data
#[update] 
fn get_form_metadata( 
    index: u16 
) -> Result<FormMetadata, String> {

    FORM_DATA.with(|form_list| {
        let form_list =  form_list.borrow();
        let form_data = form_list.get(&index).ok_or("no data for this index".to_string())?
        .to_owned();
    
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


// #[update]
// async fn all_canister_create(name: String, desc: String) -> Result<CanisterIds, String> {
 
//     let user = caller();
//     // if !is_controller(&user) {
//     //     return Err("Unauthorised user".to_string());
//     // } 

//     let settings = CanisterSettings::default();
//     let create_arg = CreateCanisterArgument{
//         settings: Some(settings)
//     };

//     let wasms = WASM_STORE.with(|wasms| {
//         wasms.borrow().to_owned()
//     });

//     let (canister_id_1,): (CanisterIdRecord,) = match call_with_payment(
//         Principal::management_canister(), // Management canister address
//         "create_canister", // Function name
//         (create_arg,), // Argument tuple
//         2_000_000_000_000, // Payment amount in cycles
//     ).await {
//         Ok(x) => x,
//         Err((_, _)) => (CanisterIdRecord { canister_id: Principal::anonymous() },),
//     };

//     if canister_id_1.canister_id == Principal::anonymous() {
//         return Err("error creating asset canister".to_string());
//     } 

//     let install_arg = Some(AssetCanisterArgs::InitArgs);
//     // let arg_vec = install_arg.as_bytes()

//     let serialized_bytes: Vec<u8> = match install_arg {
//         // Some(install_args) => serde_json::to_string(&install_arg).unwrap().as_bytes().to_vec(),
//         Some(install_args) =>candid::encode_args((install_args,)).expect("Failed to encode arguments"),

//         None => vec![],
//     };

//     // let principal_id = new_canister_id.0.canister_id;
//     let asset_canister_id = canister_id_1.canister_id;

//     // pub const WASM: &[u8] =
//     //     include_bytes!("../assetstorage.wasm.gz");
    
//     // let wasm_file = WASM.to_vec();

//     let asset_wasm = wasms.asset_wasm_blob;

//     // create installCodeArgument
//     let install_config = InstallCodeArgument {
//         mode: CanisterInstallMode::Install,
//         wasm_module: asset_wasm,
//         canister_id: asset_canister_id,
//         arg: (serialized_bytes),
//         // arg: {vec![]},

//     };
//     // Install the Wasm code into the new canister
//     let install_result = install_code(install_config).await;


//     match install_result {
//         Ok(_) => {}
//         Err(err) => {
//             eprintln!("Error installing code: {:?}", err);
//             return Err(err.1);
//         }
//     }

//     // create minter canister 
//     let settings = CanisterSettings::default();
//     let create_arg = CreateCanisterArgument{
//         settings: Some(settings)
//     };

//     let (canister_id_2,): (CanisterIdRecord,) = match call_with_payment(
//         Principal::management_canister(), // Management canister address
//         "create_canister", // Function name
//         (create_arg,), // Argument tuple
//         2_000_000_000_000, // Payment amount in cycles
//     ).await {
//         Ok(x) => x,
//         Err((_, _)) => (CanisterIdRecord { canister_id: Principal::anonymous() },),
//     };

//     if canister_id_2.canister_id == Principal::anonymous() {
//         return Err("error creating asset canister".to_string());
//     } 

//     // install minter canister
//     let minter_canister = canister_id_2.canister_id;

//     // pub const MINTERWASM: &[u8] =
//     //     include_bytes!("../../../../estate_dao_nft/target/wasm32-unknown-unknown/release/estate_dao_nft_backend.wasm.gz");
//     //     // include_bytes!("../../../canister_dummy/target/wasm32-unknown-unknown/release/canister_dummy_backend.wasm");

//     // let wasm_file = MINTERWASM.to_vec();

//     let minter_wasm = wasms.minter_wasm_blob;

//     // create installCodeArgument
//     let install_config = InstallCodeArgument {
//         mode: CanisterInstallMode::Install,
//         wasm_module: minter_wasm,
//         canister_id: minter_canister,
//         arg: vec![],
//     };

//     // Install the Wasm code into the new canister
//     let install_result = install_code(install_config).await;

//     match install_result {
//         Ok(_) => {}
//         Err(err) => {
//             eprintln!("Error installing code: {:?}", err);
//             return Err(err.1);
//         }
//     }

//     //remove
//     let mut e:String = String::from("");

//     let res =  call(minter_canister, "init_collection", (name, desc, user), ).await; 
//         match res{
//             Ok(r) => {
//                 let (res,): (Result<String, String>,) = r;
//             }, 
//         Err(_) =>{e=String::from("error")}
//     }

//     if e == "error".to_string(){
//         return Err("error initializing struct".to_string());
//     }

//     let canister_id_data = CanisterIds{
//         asset_canister: asset_canister_id,
//         minter_canister
//     };

//     CANISTER_STORE.with(|canister_store| {
//         let mut canister_map =  canister_store.borrow_mut();
//         canister_map.insert(minter_canister.clone(), canister_id_data.clone());
//     });

//     return Ok(canister_id_data) ;
// }

//test  
#[update]
async fn test_auth_user() -> Result<Vec<Principal>, String> {

    let caller = caller();
    let mut minter_canister_vec: Vec<Principal> = Vec::new();
    minter_canister_vec.push(caller);

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

#[query]
fn get_all_canisters() -> Result<Vec<CanisterIds>, String> {

    CANISTER_STORE.with(|canister_store| {
        let mut canister_vec: Vec<CanisterIds> = Vec::new();

        let canister_map = canister_store.borrow().to_owned();
        if canister_map.is_empty() {
            return Ok(canister_vec);
        }
        for (_key, value) in canister_map.iter() {
            canister_vec.push(value.to_owned());
        }
        return Ok(canister_vec);  
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
async fn approve_collection(index: u16, approval: bool) -> Result<CanisterIds, String> {

    let user = caller();
    // if !is_controller(&user) {
    //     return Err("Unauthorised user".to_string());
    // } 

    let form_list = FORM_DATA.with(|form_list_map| {
        form_list_map.borrow().clone()
    });

    let _form_data = form_list.get(&index)
        .ok_or("no form data for the index")?;

    if approval {

        let wasms = WASM_STORE.with(|wasms| {
            wasms.borrow().to_owned()
        });

        let settings = CanisterSettings::default();
        let create_arg = CreateCanisterArgument{
            settings: Some(settings)
        };

        let (canister_id_1,): (CanisterIdRecord,) = match call_with_payment(
            Principal::management_canister(), // Management canister address
            "create_canister", // Function name
            (create_arg,), // Argument tuple
            2_000_000_000_000, // Payment amount in cycles
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
        let settings = CanisterSettings::default();
        let create_arg = CreateCanisterArgument{
            settings: Some(settings)
        };

        let (canister_id_2,): (CanisterIdRecord,) = match call_with_payment(
            Principal::management_canister(), // Management canister address
            "create_canister", // Function name
            (create_arg,), // Argument tuple
            2_000_000_000_000, // Payment amount in cycles
        ).await {
            Ok(x) => x,
            Err((_, _)) => (CanisterIdRecord { canister_id: Principal::anonymous() },),
        };

        if canister_id_2.canister_id == Principal::anonymous() {
            return Err("error creating asset canister".to_string());
        } 

        // install minter canister
        let minter_canister = canister_id_2.canister_id;

        let minter_wasm = wasms.minter_wasm_blob;

        // create installCodeArgument
        let install_config = InstallCodeArgument {
            mode: CanisterInstallMode::Install,
            wasm_module: minter_wasm,
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

        let form_data = FORM_DATA.with(|form_list_map| {
           let form_list = form_list_map.borrow();
           form_list.get(&index).unwrap().clone()
        });

        let res =  call(minter_canister, "init_collection", (form_data,), ).await; 
            match res{
                Ok(r) => {
                    let (res,): (Result<String, String>,) = r;
                }, 
            Err(_) =>{e=String::from("error")}
        }

        if e == "error".to_string(){
            return Err("error initializing struct".to_string());
        }

        
        FORM_DATA.with(|form_list_map| {
            let mut form_list = form_list_map.borrow_mut();

            let _form_entry = form_list.remove(&index);
        });

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
    else {

        FORM_DATA.with(|form_list_map| {
            let mut form_list = form_list_map.borrow_mut();

            let _form_entry = form_list.remove(&index);
            // Ok(CanisterIds { asset_canister: (), minter_canister: () })
            Err("collection rejected".to_string())
        })
    }
}


// //collection specific data
// // #[update(guard = "allow_only_authorized_principal")] 
// #[update] 
// fn init_form_prop_details( 
//     name: String,
//     form_input: PropertyDetails
// ) -> Result<String, String> {

//     // let form_data: PropertyDetails = serde_json::from_slice(&form_input).unwrap();
//     // let counter = COUNTER.with(|counter| {
//     //     *counter.borrow_mut() += 1;
//     //     *counter.borrow()
//     // });

//     let key = name + &"propdetails";
//     FORM_PROP_DETAILS.with(|form_list| {
//         let mut form_list =  form_list.borrow_mut();
//         form_list.insert(key, form_input);
//     });

//     Ok("property details set succesfully".to_string())
// }

// //collection specific data
// // #[update(guard = "allow_only_authorized_principal")] 
// #[update] 
// fn init_form_financial_details( 
//     name: String,
//     form_input: FinancialDetails
// ) -> Result<String, String> {

//     // let form_data: PropertyDetails = serde_json::from_slice(&form_input).unwrap();
//     // let counter = COUNTER.with(|counter| {
//     //     *counter.borrow_mut() += 1;
//     //     *counter.borrow()
//     // });

//     let key = name + &"financialdetails";
//     FORM_FINANCIAL_DETAILS.with(|form_list| {
//         let mut form_list =  form_list.borrow_mut();
//         form_list.insert(key, form_input);
//     });

//     Ok("financial details set succesfully".to_string())
// }

// //collection specific data
// // #[update(guard = "allow_only_authorized_principal")] 
// #[update] 
// fn init_form_market_details( 
//     name: String,
//     form_input: MarketDetails
// ) -> Result<String, String> {

//     // let form_data: PropertyDetails = serde_json::from_slice(&form_input).unwrap();
//     // let counter = COUNTER.with(|counter| {
//     //     *counter.borrow_mut() += 1;
//     //     *counter.borrow()
//     // });

//     let key = name + &"marketdetails";
//     FORM_MARKET_DETAILS.with(|form_list| {
//         let mut form_list =  form_list.borrow_mut();
//         form_list.insert(key, form_input);
//     });

//     Ok("market details set succesfully".to_string())
// }



// //collection specific data
// // #[update(guard = "allow_only_authorized_principal")] 
// #[update] 
// fn init_form_documents( 
//     name: String,
//     form_input: Vec<HashMap<String, String>>
// ) -> Result<String, String> {

//     // let form_data: PropertyDetails = serde_json::from_slice(&form_input).unwrap();
//     // let counter = COUNTER.with(|counter| {
//     //     *counter.borrow_mut() += 1;
//     //     *counter.borrow()
//     // });

//     let key: String = name + &"document";
//     FORM_DOCUMENTS.with(|form_list| {
//         let mut form_list =  form_list.borrow_mut();
//         form_list.insert(key, form_input);
//     });

//     Ok("documents set succesfully".to_string())
// }


#[update] 
fn init_form_metadata( 
    form_input: FormMetadata
) -> Result<String, String> {

    FORM_DATA.with(|coll_data| {

        // let form_data: FormMetadata = serde_json::from_slice(&form_input).unwrap();
        let counter = COUNTER.with(|counter| {
            *counter.borrow_mut() += 1;
            *counter.borrow()
        });


        FORM_DATA.with(|form_list| {
            let mut form_list =  form_list.borrow_mut();
            form_list.insert(counter, form_input);
        });
    
        Ok("form initiated succesfully".to_string())
    })
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

// Enable Candid export
ic_cdk::export_candid!();
