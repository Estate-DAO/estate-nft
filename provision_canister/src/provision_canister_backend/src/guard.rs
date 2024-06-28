use crate::CANISTER_DATA;
use ic_cdk::caller;

pub fn caller_is_authorized_principal() -> Result<(), String> {
    let res = CANISTER_DATA.with_borrow(|canister_data| {
        canister_data
            .config
            .authorised_principals
            .contains(&caller())
    });

    if res {
        Ok(())
    } else {
        Err("Unauthorized".into())
    }
}
