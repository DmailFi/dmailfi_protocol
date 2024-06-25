use std::str::FromStr;

use candid::{candid_method, Nat, Principal};
use dmailfi_types::{
    EcdsaKeyIds, InboxData, Ledger, LedgerConfiguration, LedgerInfo, Mail, MailError, MailReply,
    Newsletter, OutgoingMail, RegistryError, SenderChannel, CORELATION_ID, EMAIL_ADDRESS,
    LOOKUP_DOMAIN_CALL_PAYMENT, MAIL_ID, NEWSLETTER_ID, SUBMIT_CALL_PAYMENT,
};
use email_address::EmailAddress;
use ic_cdk::{
    api::{
        self,
        call::{self, arg_data, RejectionCode},
        is_controller,
        management_canister::{
            self,
            ecdsa::{sign_with_ecdsa, SignWithEcdsaArgument},
            http_request::{
                self, http_request, CanisterHttpRequestArgument, HttpHeader, HttpMethod,
                HttpResponse, TransformArgs, TransformContext, TransformFunc,
            },
        },
    },
    caller, id, init, query, update,
};

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
    let (args,) = arg_data::<(Option<LedgerConfiguration>,)>();
    ledger::with_mut(|ledger| {
        if args.is_some() {
            ledger.init(args.unwrap())
        }
    })
}

#[query]
#[candid_method(query)]
async fn get_info() -> LedgerInfo {
    ledger::with(|l| l.get_info())
}

#[update(guard = "is_custodian")]
#[candid_method(update)]
async fn set_info(info: LedgerInfo) {
    ledger::with_mut(|l| l.set_info(info))
}

#[update]
#[candid_method(update)]
async fn submit_reply(corelation_id: CORELATION_ID, reply: MailReply) -> Result<(), MailError> {
    check_payment(SUBMIT_CALL_PAYMENT).unwrap();
    let email_addr_rslt = EmailAddress::from_str(&reply.sender_address).or(Err(
        MailError::GeneralError("Bad email address".to_string()),
    ))?;

    let domain = email_addr_rslt.domain();
    let register_id =
        Principal::from_text(ledger::with(|ledger| ledger.get_registry_address())).unwrap();
    let (lookup_response,): (Result<String, RegistryError>,) =
        ic_cdk::api::call::call_with_payment(
            register_id,
            "lookup_domain_name",
            (domain,),
            LOOKUP_DOMAIN_CALL_PAYMENT,
        )
        .await
        .unwrap();
    if lookup_response.is_err() {
        return Err(MailError::GeneralError(
            "Could not verify domain name".to_string(),
        ));
    }

    let domain_principal = Principal::from_text(lookup_response.unwrap()).unwrap();

    if caller() != domain_principal {
        return Err(MailError::NotAuthorized);
    }

    ledger::with_mut(|ledger| {
        let rslt = ledger.store_reply(corelation_id, reply);
        accept_payment(SUBMIT_CALL_PAYMENT);
        rslt
    })
}

// Should be called by a caister on user behalf.
#[update]
#[candid_method(update)]
async fn submit_mail(mut mail: Mail) -> Result<(), MailError> {
    let custodian_rslt = is_custodian();
    if custodian_rslt.is_err() && ic_cdk::api::call::msg_cycles_available() < SUBMIT_CALL_PAYMENT {
        return Err(MailError::GeneralError("Not Enough Cycles".to_string()));
    }

    if custodian_rslt.is_err() {
        mail.header.sender_channel = Some(SenderChannel::ICP.to_string());
        mail.header.sender_canister_id = Some(caller().to_text());
        mail.header.receipient_canister_id = Some(id().to_text())
    }

    let (mail_id_hex,) = ic_cdk::api::management_canister::main::raw_rand()
        .await
        .or(Err(MailError::FailedToGenerateMailId))?;

    let mail_id = hex::encode(mail_id_hex);
    ledger::with_mut(|ledger| {
        ledger.submit_mail(mail, mail_id)?;
        ic_cdk::api::call::msg_cycles_accept(SUBMIT_CALL_PAYMENT);
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

#[update(guard = "is_one_of_user")]
#[candid_method(update)]
async fn send_reply(correlation_id: CORELATION_ID, mut reply: MailReply) -> Result<(), MailError> {
    let user_email = ledger::with(|ledger| ledger.get_user_address(caller()))
        .ok_or(MailError::NoUserAddressFound)?;

    reply.sender_address = user_email.clone();

    let domain_vec = ledger::with_mut(|ledger| {
        let mail_id = ledger
            .corelation_map
            .get(&correlation_id)
            .ok_or(MailError::MailNotFound)?;
        let mail = ledger.mails.get(mail_id).ok_or(MailError::MailNotFound)?;
        let mut domain_vec = vec![];
        if mail.header.from == user_email {
            let local_vec: Vec<String> = mail
                .header
                .to
                .iter()
                .map(|f| {
                    let binding = EmailAddress::from_str(f).unwrap();
                    let domain = binding.domain();
                    return domain.to_string();
                })
                .collect();
            domain_vec.extend(local_vec)
        } else if mail.header.to.contains(&user_email) {
            let email_addr = EmailAddress::from_str(&user_email).unwrap();

            domain_vec.push(email_addr.domain().to_string())
        }

        if domain_vec.len() == 0 {
            return Err(MailError::NotAuthorized);
        }

        Ok(domain_vec)
    })?;

    let platform_domain = ledger::with(|ledger| ledger.get_domain_name());
    let registry_id = ledger::with(|ledger| ledger.get_registry_address());

    let registry_id = Principal::from_text(registry_id).unwrap();

    for domain in domain_vec {
        if domain == platform_domain {
            ledger::with_mut(|ledger| ledger.store_reply(correlation_id.clone(), reply.clone()))?;
            continue;
        }

        let lookup_response: Result<(Result<String, String>,), (RejectionCode, String)> =
            ic_cdk::api::call::call_with_payment(
                registry_id,
                "lookup_domain_name",
                (domain.clone(),),
                LOOKUP_DOMAIN_CALL_PAYMENT,
            )
            .await;

        if lookup_response.is_err() {
            continue;
        }

        let (l_reply,) = lookup_response.unwrap();

        if l_reply.is_err() {
            continue;
        }

        let dmailfi_canister = Principal::from_text(l_reply.unwrap()).unwrap();
        let dmailfi_response: Result<(Result<(), MailError>,), (RejectionCode, String)> =
            ic_cdk::api::call::call_with_payment(
                dmailfi_canister,
                "submit_reply",
                (correlation_id.clone(), reply.clone()),
                SUBMIT_CALL_PAYMENT,
            )
            .await;
        if dmailfi_response.is_err() {
            continue;
        }
    }

    ledger::with_mut(|ledger| ledger.store_reply(correlation_id, reply))
}

#[update(guard = "is_one_of_user")]
#[candid_method(update)]
async fn send_mail(mut mail: Mail) -> Result<(), MailError> {
    ledger::with(|ledger| {
        // this has already been checked by "is_one_of_user" guard function
        let user_address = ledger.get_user_address(caller()).unwrap();
        mail.header.from = user_address;
    });
    let platform_domain = ledger::with(|ledger| ledger.get_domain_name());

    let registry_id = ledger::with(|ledger| ledger.get_registry_address());

    let registry_id = Principal::from_text(registry_id).unwrap();

    let domain_vec = Ledger::get_receipients_domains(&mail);
    let mut failed_domain = vec![];
    let correlation_id = generate_random_id().await?;

    ledger::with_mut(|ledger| {
        mail.correlation_id = Some(correlation_id.clone());
        // Correlation Id serves as the Mail ID in this CASE.
        ledger.store_mail(mail.clone(), correlation_id.clone());
        let user_addr = ledger.get_user_address(caller());
        if user_addr.is_none() {
            return;
        }

        ledger.add_to_sent(correlation_id, user_addr.unwrap());
    });

    for domain in domain_vec {
        // let mx = mail.clone();

        if domain == platform_domain {
            let mail_id = generate_random_id().await?;
            ledger::with_mut(|ledger| {
                mail.header.receipient_canister_id = Some(id().to_text());
                let result = ledger.submit_mail(mail.clone(), mail_id);
                if result.is_err() {
                    failed_domain.push(domain.clone())
                }
            });

            continue;
        }

        let lookup_response: Result<(Result<String, String>,), (RejectionCode, String)> =
            ic_cdk::api::call::call_with_payment(
                registry_id,
                "lookup_domain_name",
                (domain.clone(),),
                LOOKUP_DOMAIN_CALL_PAYMENT,
            )
            .await;

        if lookup_response.is_err() {
            failed_domain.push(domain.clone());
            continue;
        }

        let (reply,) = lookup_response.unwrap();

        if reply.is_err() {
            let mail_id = generate_random_id().await?;
            let out_mail = OutgoingMail {
                id: mail_id,
                header: mail.header.clone(),
                body: mail.body.clone(),
            };
            send_http_mail(out_mail).await?;
            //TODO send HTTP MTA
            continue;
        }

        let dmailfi_canister = Principal::from_text(reply.unwrap()).unwrap();
        let dmailfi_response: Result<(Result<(), MailError>,), (RejectionCode, String)> =
            ic_cdk::api::call::call_with_payment(
                dmailfi_canister,
                "submit_mail",
                (mail.clone(),),
                SUBMIT_CALL_PAYMENT,
            )
            .await;
        if dmailfi_response.is_err() {
            failed_domain.push(domain.clone());
            continue;
        }

        let (reply,) = dmailfi_response.unwrap();
        if reply.is_err() {
            let err = reply.unwrap_err();
            failed_domain.push(format!("Domain: {} with error: {}", domain.clone(), err));
        }
    }

    if failed_domain.len() > 0 {
        let domains = failed_domain.join(",");
        return Err(MailError::MailTransferError(format!(
            "The following domains {} failed",
            domains
        )));
    }
    Ok(())
}

#[query]
fn transform(args: TransformArgs) -> HttpResponse {
    let mut res = http_request::HttpResponse {
        status: args.response.status.clone(),
        body: args.response.body.clone(),
        headers: vec![],
    };

    if res.status == Nat::from(200u32) {
        res.body = args.response.body;
    } else {
        ic_cdk::api::print(format!(
            "Received an error from http server: err = {:?}",
            args
        ));
    }
    res
}

async fn generate_random_id() -> Result<String, MailError> {
    let (mail_id_hex,) = ic_cdk::api::management_canister::main::raw_rand()
        .await
        .or(Err(MailError::FailedToGenerateMailId))?;
    let mail_id = hex::encode(mail_id_hex);
    Ok(mail_id)
}

async fn send_http_mail(out: OutgoingMail) -> Result<(), MailError> {
    let mta_url = ledger::with(|ledger| ledger.get_mail_transfer_agent_url());
    let sig = sign_data(&out.id)
        .await
        .map_err(|err| MailError::HttpSendMail(err))?;
    let canister_id = api::id().to_text();
    let headers = vec![
        HttpHeader {
            name: "x-sig".to_string(),
            value: sig,
        },
        HttpHeader {
            name: "x-principal".to_string(),
            value: canister_id,
        },
    ];
    let out_json = serde_json::to_string(&out).unwrap();
    let request = CanisterHttpRequestArgument {
        url: mta_url,
        max_response_bytes: Some(256),
        method: HttpMethod::POST,
        headers,
        body: Some(out_json.into_bytes()),
        transform: Some(TransformContext {
            function: TransformFunc(candid::Func {
                principal: ic_cdk::api::id(),
                method: "transform".to_string(),
            }),
            context: vec![],
        }),
    };

    match http_request(request, 20_000_000_000).await {
        Ok((resp,)) => Ok(()),
        Err((code, mssg)) => Err(MailError::HttpSendMail(mssg)),
    }
}

fn sha256(input: &str) -> [u8; 32] {
    use sha2::Digest;
    let mut hasher = sha2::Sha256::new();
    hasher.update(input);
    hasher.finalize().into()
}

async fn sign_data(mssg_hash: &String) -> Result<String, String> {
    let data = management_canister::ecdsa::sign_with_ecdsa(SignWithEcdsaArgument {
        message_hash: sha256(mssg_hash).to_vec(),
        derivation_path: vec![b"asymetric".to_vec()],
        #[cfg(network = "ic")]
        key_id: EcdsaKeyIds::ProductionKey1.to_key_id(),
        #[cfg(network = "local")]
        key_id: EcdsaKeyIds::TestKeyLocalDevelopment.to_key_id(),
    })
    .await;

    match data {
        Ok((resp,)) => Ok(hex::encode(resp.signature)),

        Err(err) => Err(err.1),
    }
}

#[update(guard = "is_custodian")]
#[candid_method(update)]
async fn send_newsletter(n_id: NEWSLETTER_ID, mail: Mail) -> Result<(), MailError> {
    let emails = ledger::with(|ledger| ledger.get_newsletter_subscribers(n_id))?;

    for addr in emails {
        let mut mx = mail.clone();
        mx.header.to = vec![addr];
        send_mail(mx).await;
    }

    Ok(())
}

#[query]
#[candid_method(query)]
async fn exchange_key() -> String {
    //TODO (IMplement Exchange Key)
    todo!()
}

#[update]
#[candid_method(update)]
async fn subscribe_to_newsletter(
    addr: EMAIL_ADDRESS,
    n_id: NEWSLETTER_ID,
) -> Result<(), MailError> {
    ledger::with_mut(|ledger| ledger.subscribe_to_newsletter(n_id, addr, caller()))
}

#[update]
#[candid_method(update)]
async fn unsubscribe_to_newsletter(
    addr: EMAIL_ADDRESS,
    n_id: NEWSLETTER_ID,
) -> Result<(), MailError> {
    ledger::with_mut(|ledger| ledger.unsubscribe_to_newsletter(n_id, addr, caller()))
}

#[update(guard = "is_custodian")]
#[candid_method(update)]
async fn create_newsletter(n: Newsletter) -> Result<(), MailError> {
    let (n_id_hex,) = ic_cdk::api::management_canister::main::raw_rand()
        .await
        .or(Err(MailError::FailedToGenerateMailId))?;
    let n_id = hex::encode(n_id_hex);

    ledger::with_mut(|ledger| ledger.create_newletter(n_id, n))
}

#[query]
#[candid_method(query)]
async fn get_newsletters() -> std::vec::Vec<(std::string::String, Newsletter)> {
    ledger::with(|ledger| ledger.get_newsletters())
}

#[query]
#[candid_method(query)]
async fn get_newsletter(n_id: NEWSLETTER_ID) -> Result<Newsletter, MailError> {
    ledger::with(|ledger| ledger.get_newsletter(n_id))
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

fn check_payment<'a>(payment: u64) -> Result<(), &'a str> {
    if ic_cdk::api::call::msg_cycles_available() < payment {
        return Err("Insufficient Fund");
    }

    Ok(())
}

fn accept_payment(payment: u64) {
    ic_cdk::api::call::msg_cycles_accept(payment);
}

fn not_anonymous() -> Result<(), String> {
    if caller() == Principal::anonymous() {
        return Err("User is anonymous".to_string());
    } else {
        return Ok(());
    }
}

fn is_one_of_user() -> Result<(), String> {
    not_anonymous()?;
    let address = ledger::with(|ledger| ledger.get_user_address(caller()));
    if address.is_none() {
        return Err("There's no email address associated with this prinicpal".to_string());
    } else {
        return Ok(());
    }
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
