
use candid::{candid_method, encode_args, Principal};
use dmailfi_types::{LedgerConfiguration, MailError, Rcbytes, RegistryError, LOOKUP_DOMAIN_CALL_PAYMENT};
use ic_cdk::{
    api::{is_controller, management_canister::{
        self, main::{CanisterInstallMode, CreateCanisterArgument, InstallCodeArgument}, provisional::CanisterSettings
    }}, caller, post_upgrade, query, update
};
use ledger::DMAILFI_WASM;
use types::{DOMAIN_NAME};
mod types;
mod ledger {
    use std::{cell::RefCell, sync::Arc};

    use dmailfi_types::Rcbytes;

    use crate::types::Ledger;

    thread_local!(
        static LEDGER: RefCell<Ledger> = RefCell::new(Ledger::default());
        pub static DMAILFI_WASM : RefCell<Rcbytes> =  RefCell::new(Rcbytes::new(Arc::new(serde_bytes::ByteBuf::from(include_bytes!("dmailfi_core.wasm")))))
    );

    pub fn with<T, F: FnOnce(&Ledger) -> T>(f: F) -> T {
        LEDGER.with(|ledger| f(&ledger.borrow()))
    }

    pub fn with_mut<T, F: FnOnce(&mut Ledger) -> T>(f: F) -> T {
        LEDGER.with(|ledger| f(&mut ledger.borrow_mut()))
    }
}

#[ic_cdk::query]
fn greet(name: String) -> String {
    format!("Hello, {}!", name)
}

#[query]
#[candid_method(query)]
async fn lookup_domain_name(
    domain_name: DOMAIN_NAME,
) -> Result<std::string::String, RegistryError> {
    if ic_cdk::api::call::msg_cycles_available() < LOOKUP_DOMAIN_CALL_PAYMENT {
        return Err(RegistryError::GeneralError("Not Enough Cycles".to_string()));
    }
    let result = ledger::with(|ledger| ledger.lookup_domain_name(domain_name));
    ic_cdk::api::call::msg_cycles_accept(LOOKUP_DOMAIN_CALL_PAYMENT);
    result
}

#[update(guard = "is_custodian")]
#[candid_method(update)]
async fn create_dmail_canister(domain_name: DOMAIN_NAME, controller_principal: String, config : Option<LedgerConfiguration>) -> Result<String, RegistryError> {
    let registry_id = ic_cdk::api::id();
    let arg = CreateCanisterArgument {
        settings: Some(CanisterSettings {
            controllers: Some(vec![
                Principal::from_text(controller_principal.clone()).unwrap(),
                registry_id,
            ]),
            ..CanisterSettings::default()
        }),
    };
    let (cr,) = ic_cdk::api::management_canister::main::create_canister(arg, 9_000_000_000_000)
        .await
        .map_err(|_| RegistryError::FailedToCreateCanister)?;

    let Rcbytes(wasm) = DMAILFI_WASM.with_borrow(|f| f.clone());

    let installation_result = ic_cdk::api::management_canister::main::install_code(InstallCodeArgument {
        mode: ic_cdk::api::management_canister::main::CanisterInstallMode::Install,
        canister_id: cr.canister_id,
        wasm_module: wasm.to_vec(),
        arg: encode_args((config, )).unwrap(),
    }).await;

    if installation_result.is_err() {
        ledger::with_mut(|ledger|{
            ledger.add_to_pending_canister(cr.canister_id, Principal::from_text(controller_principal).unwrap());
        });

        return Err(RegistryError::FailedToInstallCode(cr.canister_id.to_string()));
    }

    ledger::with_mut(|ledger| {
        ledger.add_domain(domain_name, cr.canister_id.to_text())
    });

    Ok(cr.canister_id.to_string())

}

#[update(guard = "is_custodian")]
#[candid_method(update)]
async fn upgrade_all_dmail_canisters() -> Result<(), RegistryError> {
    let canister_ids = ledger::with(|ledger| {
        ledger.get_all_domain_canisters()
    });

    let Rcbytes(wasm) = DMAILFI_WASM.with_borrow(|f| f.clone());
    let mut failed_domains = vec![];
    for canister_id in canister_ids {
        let canister_principal = Principal::from_text(canister_id.clone()).unwrap();
        let arg = InstallCodeArgument { mode: CanisterInstallMode::Upgrade, canister_id: canister_principal, wasm_module: wasm.to_vec(), arg: vec![] };
        let reslt = management_canister::main::install_code(arg).await;
        if reslt.is_err() {
            failed_domains.push(canister_id)
        }
    }

    if failed_domains.len() > 0 {
        let word = failed_domains.join(",");
        return Err(RegistryError::FailedToUpgrade(format!("These canisters {} failed to be upgrade", word)));
    }

    Ok(())
}

#[query]
#[candid_method(query)]
async fn get_domain_details(domain_name: DOMAIN_NAME) -> Result<std::string::String, RegistryError> {
    ledger::with(|ledger| ledger.get_domain_details(domain_name))
}

#[query(guard = "not_anonymous")]
#[candid_method(query)]
async fn lookup_user() -> Result<Vec<String>, RegistryError> {
    ledger::with(|ledger| ledger.lookup_user(caller()))
}

fn is_custodian() -> Result<(), std::string::String> {
    if is_controller(&caller()) {
        return Ok(());
    }
    ledger::with(|ledger|{
        ledger.is_custodian(caller().to_text())
    })
}

fn not_anonymous() -> Result<(), std::string::String> {
    if caller() == Principal::anonymous() {
        Err("You are anonymous".to_string())
    } else {
        Ok(())
    }
}

#[query]
#[candid_method(query)]
fn export_candid() -> String {
    ic_cdk::export_candid!();
    __export_service()
}
