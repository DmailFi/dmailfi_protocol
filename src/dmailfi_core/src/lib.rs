use candid::{candid_method, Principal};
use ic_cdk::{api::call::{arg_data, RejectionCode}, caller, init, query, update};
use dmailfi_types::{InboxData, Ledger, LedgerConfiguration, Mail, MailError, Newsletter, EMAIL_ADDRESS, MAIL_ID, NEWSLETTER_ID};

pub mod ledger {
    use std::cell::RefCell;

    use dmailfi_types::Ledger;

    thread_local!(
        static LEDGER: RefCell<Ledger> = RefCell::new(Ledger::default());
    );

    pub fn with<T, F: FnOnce(&Ledger) -> T>(f: F) -> T {
        LEDGER.with(|ledger| f(&ledger.borrow()))
    }

    pub fn with_mut<T, F: FnOnce(&mut Ledger) -> T>(f: F) -> T {
        LEDGER.with(|ledger| f(&mut ledger.borrow_mut()))
    }
}


#[init]
#[candid_method(init)]
fn init() {
    let (args, ) = arg_data::<(Option<LedgerConfiguration>,)>();
    ledger::with_mut(|ledger| {
        if args.is_some() {
            ledger.init(args.unwrap())
        }
    })
}


#[update]
#[candid_method(update)]
async fn submit_mail(mail: Mail) -> Result<(), MailError> {
    let (mail_id_hex,) = ic_cdk::api::management_canister::main::raw_rand()
        .await
        .or(Err(MailError::FailedToGenerateMailId))?;
    let mail_id = hex::encode(mail_id_hex);
    ledger::with_mut(|ledger| {
        ledger.submit_mail(mail, mail_id)?;
        Ok(())
    })
}

#[update]
#[candid_method(update)]
async fn get_mail(mail_id: MAIL_ID) -> Result<Mail, MailError> {
    ledger::with_mut(|ledger| ledger.get_mail(mail_id))
}

#[query]
#[candid_method(query)]
async fn get_users() -> Result<std::vec::Vec<std::string::String>, MailError> {
    ledger::with(|ledger| ledger.get_users())
}

#[query]
#[candid_method(query)]
async fn get_mails(page: Option<usize>) -> Result<std::vec::Vec<InboxData>, MailError> {
    ledger::with(|ledger| ledger.get_mails(page))
}

#[query]
#[candid_method(query)]
async fn get_all_mail_count() -> Result<(u32, u32), MailError> {
    ledger::with(|ledger| ledger.get_all_mail_count())
}

#[update(guard = "is_custodian")]
#[candid_method[update]]
async fn create_user(
    email_address: EMAIL_ADDRESS,
    principal_address: String,
) -> Result<(), MailError> {
    ledger::with_mut(|ledger| ledger.create_user(email_address, principal_address))
}

#[update(guard = "is_custodian")]
#[candid_method[update]]
async fn delete_user(email_address: EMAIL_ADDRESS) -> Result<(), MailError> {
    ledger::with_mut(|ledger| ledger.delete_user(email_address))
}

#[query]
#[candid_method[query]]
async fn get_mail_count() -> Result<(u32, u32), MailError> {
    ledger::with(|ledger| ledger.get_mail_count())
}

#[update]
#[candid_method[update]]
async fn delete_mail(mail_id: MAIL_ID) -> Result<(), MailError> {
    ledger::with_mut(|ledger| ledger.delete_mail(mail_id))
}

#[update]
#[candid_method[update]]
async fn restore_mail(mail_id: MAIL_ID) -> Result<(), MailError> {
    ledger::with_mut(|ledger| ledger.restore_mail(mail_id))
}

#[query]
#[candid_method(query)]
async fn get_domain_name() -> std::string::String {
    ledger::with(|ledger| ledger.get_domain_name())
}

#[query]
#[candid_method(query)]
async fn get_token_name() -> std::string::String {
    ledger::with(|ledger| ledger.get_token_address())
}

#[update]
#[candid_method(update)]
async fn send_mail(mail: Mail) -> Result<(), MailError> {
    let platform_domain = ledger::with(|ledger| ledger.get_domain_name());

    let registry_id = ledger::with(|ledger| ledger.get_registry_address());

    let registry_id = Principal::from_text(registry_id).unwrap();

    let domain_vec = Ledger::get_domains(&mail);
    let mut failed_domain = vec![];
    for domain in domain_vec {
        // let mx = mail.clone();
        if domain == platform_domain {
            let (mail_id_hex,) = ic_cdk::api::management_canister::main::raw_rand()
                .await
                .or(Err(MailError::FailedToGenerateMailId))?;
            let mail_id = hex::encode(mail_id_hex);

            ledger::with_mut(|ledger| {
                let result = ledger.submit_mail(mail.clone(), mail_id);
                if result.is_err() {
                    failed_domain.push(domain.clone())
                }
            });
        }

        let lookup_response: Result<(Result<String, String>,), (RejectionCode, String)> =
            ic_cdk::call(registry_id, "lookup_domain_name", (domain.clone(),)).await;

        if lookup_response.is_err() {
            failed_domain.push(domain.clone());
            continue;
        }

        let (reply,) = lookup_response.unwrap();

        if reply.is_err() {
            //TODO send HTTP MTA
            continue;
        }

        let dmailfi_canister = Principal::from_text(reply.unwrap()).unwrap();
        let dmailfi_response: Result<(Result<(), MailError>,), (RejectionCode, String)> =
            ic_cdk::call(dmailfi_canister, "submit_mail", (mail.clone(),)).await;
        if dmailfi_response.is_err() {
            failed_domain.push(domain.clone());
            continue;
        }

        let (reply,) = dmailfi_response.unwrap();
        if reply.is_err() {
            failed_domain.push(domain.clone());
        }
    };

    if failed_domain.len() > 0 {
        let domains = failed_domain.join(",");
        return Err(MailError::MailTransferError(format!("The following domains {} failed", domains)));
    }
    Ok(())
}

#[update(guard = "is_custodian")]
#[candid_method(update)]
async fn send_newsletter(n_id : NEWSLETTER_ID,mail: Mail) -> Result<(), MailError> {
    let emails = ledger::with(|ledger|{
        ledger.get_newsletter_subscribers(n_id)
    })?;

    for addr in emails {
        let mut mx = mail.clone();
        mx.header.to = vec![addr];
        send_mail(mx).await;
    };

    Ok(())
}

#[update]
#[candid_method(update)]
async fn subscribe_to_newsletter(addr : EMAIL_ADDRESS, n_id: NEWSLETTER_ID) -> Result<(), MailError> {
    ledger::with_mut(|ledger|{
        ledger.subscribe_to_newsletter(n_id, addr, caller())
    })
}

#[update]
#[candid_method(update)]
async fn unsubscribe_to_newsletter(addr : EMAIL_ADDRESS, n_id: NEWSLETTER_ID) -> Result<(), MailError> {
    ledger::with_mut(|ledger|{
        ledger.unsubscribe_to_newsletter(n_id, addr, caller())
    })
}

#[update(guard = "is_custodian")]
#[candid_method(update)]
async fn create_newsletter(n : Newsletter) -> Result<(), MailError> {
    let (n_id_hex,) = ic_cdk::api::management_canister::main::raw_rand()
        .await
        .or(Err(MailError::FailedToGenerateMailId))?;
    let n_id = hex::encode(n_id_hex);
    
    ledger::with_mut(|ledger|{
        ledger.create_newletter(n_id, n)
    })
}

#[query]
#[candid_method(query)]
async fn get_newsletters() -> std::vec::Vec<(std::string::String, Newsletter)> {
    ledger::with(|ledger|{
        ledger.get_newsletters()
    })
}

#[query]
#[candid_method(query)]
async fn get_newsletter(n_id: NEWSLETTER_ID) -> Result<Newsletter, MailError> {
    ledger::with(|ledger|{
        ledger.get_newsletter(n_id)
    })
}

#[update]
#[candid_method(update)]
async fn public_create_user(email_address: EMAIL_ADDRESS) -> Result<(), MailError> {
    ledger::with_mut(|ledger| ledger.public_create_user(email_address))
}

#[update]
#[candid_method(update)]
async fn delete_self() -> Result<(), MailError> {
    ledger::with_mut(|ledger| ledger.delete_self())
}

#[query]
#[candid_method(query)]
fn export_candid() -> String {
    ic_cdk::export_candid!();
    __export_service()
}

fn is_custodian() -> Result<(), String> {
    ledger::with(|ledger| ledger.is_custodian(caller()))
}
