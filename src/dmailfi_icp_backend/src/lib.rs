use std::fs;
use serde::Deserialize;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{transport, Message, SmtpTransport, Transport};
use candid::{candid_method, encode_args, Principal};
use dmailfi_types::{LedgerConfiguration, MailError, Rcbytes};
use ic_cdk::{
    api::management_canister::{
        self, main::{CanisterInstallMode, CreateCanisterArgument, InstallCodeArgument}, provisional::CanisterSettings
    }, caller, query, update
};
use ledger::DMAILFI_WASM;
use types::{RegistryError, DOMAIN_NAME};
mod types;
mod ledger {
    use std::{cell::RefCell, sync::Arc};

    use dmailfi_types::Rcbytes;

    use crate::types::Ledger;
    use serde_json::Error;

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
    ledger::with(|ledger| ledger.lookup_domain_name(domain_name))
}

#[update(guard = "is_custodian")]
#[candid_method(update)]
async fn create_dmail_canister(domain_name: DOMAIN_NAME, controller_principal: String, config : Option<LedgerConfiguration>) {
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
        .unwrap();

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

        return ;
    }

    ledger::with_mut(|ledger| {
        ledger.add_domain(domain_name, cr.canister_id.to_text())
    })

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

fn is_custodian() -> Result<(), std::string::String> {
    ledger::with(|ledger|{
        ledger.is_custodian(caller().to_text())
    })
}

#[query]
#[candid_method(query)]
fn export_candid() -> String {
    ic_cdk::export_candid!();
    __export_service()
}

#[derive(Deserialize, Debug)]
struct SmtpServer {
    address: String,
    port: u16,
    username: String,
    password: String,
}

// function to read SMTP server details from a configuration file
fn read_smtp_server_details() -> Result<SmtpServer, MailError> {
    let config_file = fs::read_to_string("smtp_servers.json")
        .expect("Unable to read SMTP servers configuration file");

    let servers: Vec<SmtpServer> = serde_json::from_str(&config_file)
        .expect("Unable to parse SMTP servers configuration file");

    Ok(servers[0])
}

//function t establish a connection to the SMTP server
fn connect_smtp_server(servers: Vec<SmtpServer>) -> Result<SmtpTransport, MailError> {
    let mut transports = Vec::new();
    for server in servers {
        let creds = Credentials::new(server.username, server.password);
        // let transport = SmtpClient::new_simple(&server.address)
        //     .unwrap()
        //     .credentials(creds)
        //     .transport();
        // transports.push(transport);
        let transport = transport::smtp::SmtpTransport::starttls_relay(&server.address)
            .unwrap()
            .credentials(creds)
            .build();
        transports.push(transport);
    }
    let server = SmtpServer {
        address: String::from("your_smtp_address"),
        port: 587,
        username: String::from("your_username"),
        password: String::from("your_password"),
    };

    let transport = SmtpTransport::relay(&server.address)
        .unwrap()
        .build();
    Ok(transport)
}