use std::{
    borrow::Borrow, cell::RefCell, cmp, collections::{HashMap, HashSet}, default, str::FromStr, sync::Arc
};

use candid::{types::TypeInner, CandidType, Principal};
use email_address::EmailAddress;
use ic_cdk::{api::management_canister::ecdsa::{EcdsaCurve, EcdsaKeyId}, caller};
use serde::{de::Visitor, Deserialize, Serialize};
use serde_bytes::ByteBuf;

pub type EMAIL_ADDRESS = String;
pub type MAIL_ID = String;
pub type NEWSLETTER_ID = String;

struct RcbytesVisitor;

impl<'de> Visitor<'de> for RcbytesVisitor {
    type Value = Rcbytes;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a byte array")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Rcbytes(Arc::new(ByteBuf::from(v))))
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Rcbytes(Arc::new(ByteBuf::from(v))))
    }

    

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {

        let len = cmp::min(seq.size_hint().unwrap_or(0), 4096);
        let mut bytes = Vec::with_capacity(len);

        while let Some(b) = seq.next_element()? {
            bytes.push(b)
        };

        Ok(Rcbytes(Arc::new(ByteBuf::from(bytes))))
    }

    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
        where
            E: serde::de::Error, {
        Ok(Rcbytes(Arc::new(ByteBuf::from(v))))
    }

   
    fn visit_char<E>(self, v: char) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_str(v.encode_utf8(&mut [0u8; 4]))
    }

    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_str(v)
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_str(&v)
    }

    fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_bytes(v)
    }

}
pub struct Rcbytes(pub Arc<serde_bytes::ByteBuf>);

impl Rcbytes {
    pub fn new(arc : Arc<serde_bytes::ByteBuf>) -> Self {
        Rcbytes(arc)
    }
}

impl CandidType for Rcbytes {
    fn _ty() -> candid::types::Type {
        TypeInner::Vec(TypeInner::Nat8.into()).into()
    }

    fn idl_serialize<S>(&self, serializer: S) -> Result<(), S::Error>
    where
        S: candid::types::Serializer {
       serializer.serialize_blob(&self.0)
    }
}

impl<'de> Deserialize<'de> for Rcbytes {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de> {
        deserializer.deserialize_bytes(RcbytesVisitor)
    }
}

impl Serialize for Rcbytes {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer {
            serializer.serialize_bytes(&self.0)
    }
}

impl Clone for Rcbytes {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

#[derive(CandidType, Deserialize, Clone, Default, Serialize)]
pub struct MailHeader {
    pub from: String,
    pub timestamp: u64,
    pub content_type: Option<String>,
    pub to: Vec<EMAIL_ADDRESS>,
    pub subject: Option<String>,
    pub cc: Option<Vec<String>>,
    pub bcc: Option<Vec<String>>,
}

impl Clone for Mail {
    fn clone(&self) -> Self {
        Self { header: self.header.clone(), body: self.body.clone() }
    }
}
#[derive(CandidType, Deserialize, Serialize)]
pub struct OutgoingMail {
    pub id: MAIL_ID,
    pub header: MailHeader,
    pub body:Rcbytes
}

pub enum EcdsaKeyIds {
    #[allow(unused)]
    TestKeyLocalDevelopment,
    #[allow(unused)]
    TestKey1,
    #[allow(unused)]
    ProductionKey1,
}

impl EcdsaKeyIds {
    pub fn to_key_id(&self) -> EcdsaKeyId {
        EcdsaKeyId {
            curve: EcdsaCurve::Secp256k1,
            name: match self {
                Self::TestKeyLocalDevelopment => "dfx_test_key",
                Self::TestKey1 => "test_key_1",
                Self::ProductionKey1 => "key_1",
            }
            .to_string(),
        }
    }
}


#[derive(CandidType, Deserialize)]
pub struct Mail {
    pub header: MailHeader,
    pub body: Rcbytes,
}

pub struct MailStatus {
    read : bool,
    mail_id: MAIL_ID
}

#[derive(CandidType, Deserialize, Clone)]
pub struct Newsletter {
    title : String,
    desciption : String
}

#[derive(CandidType, Deserialize)]
pub enum RegistryError {
    NotFound,
    FailedToUpgrade(String),
    FailedToCreateCanister,
    FailedToInstallCode(String)
}



pub struct Profile {
    name : String,
    portfolio: String,
    photo: Rcbytes
}


#[derive(CandidType, Deserialize)]
#[derive(Default)]
pub struct LedgerConfiguration {
    registry_canister: String,
    token_address : String,
    permissioned: bool,
    mta_url: String,
    domain_name: String,
    show_logs: bool,
    version: String
}
#[derive(Default)]
pub struct Ledger {
    custodians: HashSet<Principal>,
    users: HashMap<Principal, EMAIL_ADDRESS>,
    profile: HashMap<EMAIL_ADDRESS, Profile>,
    inboxes: HashMap<EMAIL_ADDRESS, HashSet<MAIL_ID>>,
    mail_status: HashMap<MAIL_ID, MailStatus>,
    trash: HashMap<EMAIL_ADDRESS, HashSet<MAIL_ID>>,
    mails: HashMap<MAIL_ID, Mail>,
    // audit_logs: Vec<String>,
    config: LedgerConfiguration,
    newsletter_subscribers: HashMap<NEWSLETTER_ID, HashMap<EMAIL_ADDRESS, Principal>>,
    newsletter: HashMap<NEWSLETTER_ID, Newsletter>,

}


#[derive(CandidType, Deserialize)]
pub enum MailError {
    NoUserAddressFound,
    InternalSystemMailCollision,
    FailedToGenerateMailId,
    MailNotFound,
    NotAuthorized,
    PermissionedSystem,
    AddressExist,
    DomainNotFound,
    MailTransferError(String),
    NotFound,
    HttpSendMail(String)
}

impl std::fmt::Display for MailError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MailError::NoUserAddressFound => f.write_str("No user Address Found"),
            MailError::InternalSystemMailCollision => f.write_str("A rare event of Hash map collision"),
            MailError::FailedToGenerateMailId => f.write_str("System failed to Generate Mail Id"),
            MailError::MailNotFound => f.write_str("Mail not found"),
            MailError::NotAuthorized => f.write_str("You are not authorized"),
            MailError::PermissionedSystem => f.write_str("This is a Permissioned System"),
            MailError::AddressExist => f.write_str("Address do exist"),
            MailError::DomainNotFound => f.write_str("Domain not Found"),
            MailError::MailTransferError(_) => f.write_str("Mail Transfer Error"),
            MailError::NotFound => f.write_str("Not Found"),
            MailError::HttpSendMail(_) => f.write_str("Error using internal HTTP outcall"),
        }
    }
}


#[derive(CandidType, Deserialize)]
pub struct InboxData {
    header : MailHeader,
    read: bool
}


impl Ledger {
    pub fn init(&mut self, config : LedgerConfiguration) {
        self.config = config
    }
    pub fn submit_mail(&mut self, mail: Mail, intended_mail_id: String) -> Result<(), MailError> {
        let mut selected_users = vec![];
        for user in &mail.header.to {
            if self.profile.contains_key(user) {
                selected_users.push(user.clone());
            }
        }

        if mail.header.cc.is_some() {
            for user in mail.header.cc.as_ref().unwrap() {
                if self.profile.contains_key(user) {
                    selected_users.push(user.clone());
                }
            }
        }

        if mail.header.bcc.is_some() {
            for user in mail.header.bcc.as_ref().unwrap() {
                if self.profile.contains_key(user) {
                    selected_users.push(user.clone());
                }
            }
        }

        if selected_users.len() == 0 {
            return Err(MailError::NoUserAddressFound);
        }

        if self.mails.contains_key(&intended_mail_id) {
            return Err(MailError::InternalSystemMailCollision);
        }

        for selected_user in &selected_users {
            let inbox_set = self
                .inboxes
                .get_mut(selected_user)
                .ok_or(MailError::NoUserAddressFound)?;
            inbox_set.insert(intended_mail_id.clone());
        }

        self.mail_status.insert(intended_mail_id.clone(), MailStatus { read: false, mail_id: intended_mail_id.clone() });

        self.mails.insert(intended_mail_id, mail);

        Ok(())
    }

    pub fn is_custodian(&self, principal : Principal) -> Result<(), String> {
        if self.custodians.contains(&principal) {
            Ok(())
        } else {
            Err("You are not a custodian of this canister".to_string())
        }
    }

    pub fn get_mail(&mut self, mail_id : String) -> Result<Mail, MailError> {
        let email = self.users.get(&caller()).ok_or(MailError::NoUserAddressFound)?;
        let inbox = self.inboxes.get(email).ok_or(MailError::NoUserAddressFound)?;
        if inbox.contains(&mail_id) {
            let mail = self.mails.get(&mail_id).ok_or(MailError::MailNotFound)?;
            let mailstatus = self.mail_status.get_mut(&mail_id).ok_or(MailError::MailNotFound)?;
            mailstatus.read = true;
            Ok(mail.clone())
        } else {
            Err(MailError::MailNotFound)
        }
    }

    pub fn get_mails(&self, page : Option<usize>) -> Result<Vec<InboxData>, MailError> {
        let email = self.users.get(&caller()).ok_or(MailError::NoUserAddressFound)?;
        let inbox = self.inboxes.get(email).ok_or(MailError::NoUserAddressFound)?;
        let mut inbox_data_vec = vec![];
        let mut skip = 0;
        if page.is_some() {
            let page_num = page.unwrap();
            skip = page_num * 100
        }
        for mail_id in inbox.iter().skip(skip).take(100) {
            let mail = self.mails.get(mail_id).ok_or(MailError::MailNotFound)?;
            let status = self.mail_status.get(mail_id).ok_or(MailError::MailNotFound)?;
            inbox_data_vec.push(InboxData{
                header: mail.header.clone(),
                read: status.read
            })
        }

        Ok(inbox_data_vec)
    }

    pub fn get_users(&self) -> Result<Vec<EMAIL_ADDRESS>, MailError> {
        if self.config.permissioned {
            Ok(self.users.values().cloned().collect())
        } else {
            if self.is_custodian(caller()).is_err() {
                Err(MailError::NotAuthorized)
            } else {
                Ok(self.users.values().cloned().collect())
            }
        }

    }

    pub fn get_all_mail_count(&self) -> Result<(u32, u32), MailError> {
        let mut unread = 0;
        let mut read = 0;
        for status in self.mail_status.values() {
            if status.read {
                read += 1
            } else {
                unread += 1
            }
        }

        Ok((unread, read))
    }

    pub fn get_mail_count(&self) -> Result<(u32, u32), MailError> {
        let email = self.users.get(&caller()).ok_or(MailError::NoUserAddressFound)?;
        let inbox = self.inboxes.get(email).ok_or(MailError::NoUserAddressFound)?;
        let mut unread = 0;
        let mut read = 0;
        for mail_id in inbox {
            let status = self.mail_status.get(mail_id).ok_or(MailError::MailNotFound)?;
            if status.read {
                read += 1
            } else {
                unread += 1
            }
        }

        Ok((unread, read))
    }

    pub fn create_user(&mut self, email_address : EMAIL_ADDRESS, principal_address : String) -> Result<(), MailError> {
        let user_p = Principal::from_text(principal_address).unwrap();
        self.inboxes.insert(email_address.clone(), HashSet::new());
        // self.trash.insert(email_address.clone(), HashSet::new())
        self.users.insert(user_p, email_address);
        Ok(())
    }

    pub fn public_create_user(&mut self, email_address : EMAIL_ADDRESS) -> Result<(), MailError> {
        if self.config.permissioned {
            return Err(MailError::PermissionedSystem)
        }

        if self.inboxes.contains_key(&email_address) {
            return Err(MailError::AddressExist);
        }

        self.inboxes.insert(email_address.clone(), HashSet::new());
        self.users.insert(caller(), email_address);
        Ok(())
    }

    pub fn delete_user(&mut self, email_address : EMAIL_ADDRESS) -> Result<(), MailError> {
        if !self.config.permissioned {
            return Err(MailError::NotAuthorized);
        }
        let mut px : Option<Principal> = None;
        for (principal, email) in self.users.borrow() {
            if &email_address == email {
                px = Some(principal.clone());
                break;
            }
        }

        if px.is_some() {
            self.users.remove(&px.unwrap());
        }

        Ok(())
    }

    pub fn delete_self(&mut self) -> Result<(), MailError> {
        let email = self.users.get(&caller()).ok_or(MailError::NoUserAddressFound)?;
        self.profile.remove(email);
        self.inboxes.remove(email);
        self.trash.remove(email);
        self.users.remove(&caller());

        Ok(())
    }



    pub fn delete_mail(&mut self, mail_id : String) -> Result<(), MailError> {
        let email = self.users.get(&caller()).ok_or(MailError::NoUserAddressFound)?;
        let inbox = self.inboxes.get_mut(email).ok_or(MailError::NoUserAddressFound)?;
        if inbox.contains(&mail_id) {
            inbox.remove(&mail_id);
            let mut trash_set = HashSet::new();
            trash_set.insert(mail_id);
            self.trash.insert(email.clone(),  trash_set);
        }

        Ok(())
    }

    pub fn restore_mail(&mut self, mail_id : MAIL_ID) -> Result<(), MailError> {
        let email = self.users.get(&caller()).ok_or(MailError::NoUserAddressFound)?;
        let trash_set = self.trash.get_mut(email).ok_or(MailError::MailNotFound)?;
        if trash_set.contains(&mail_id) {
            trash_set.remove(&mail_id);
            let inbox_set = self.inboxes.get_mut(email).ok_or(MailError::MailNotFound)?;
            inbox_set.insert(mail_id);
            Ok(())
        } else {
            Err(MailError::MailNotFound)
        }
    }

    pub fn get_domain_name(&self) -> String {
        self.config.domain_name.clone()
    }

    pub fn get_token_address(&self) -> String {
        self.config.token_address.clone()
    }

    pub fn get_registry_address(&self) -> String {
        self.config.registry_canister.clone()
    }

    pub fn get_domains(mail: &Mail) -> Vec<String> {
        let mut selected_domains = vec![];
        for to in &mail.header.to {
            if !EmailAddress::is_valid(to) {
                ic_cdk::trap(format!("{} is not valid", to).as_str())
            }

            let email = EmailAddress::from_str(&to).unwrap();
            let domain_name = email.domain().to_string();
            selected_domains.push(domain_name)
        }

        if mail.header.bcc.is_some() {
            for to in mail.header.bcc.as_ref().unwrap() {
                if !EmailAddress::is_valid(to) {
                    ic_cdk::trap(format!("{} is not valid", to).as_str())
                }
    
                let email = EmailAddress::from_str(&to).unwrap();
                let domain_name = email.domain().to_string();
                selected_domains.push(domain_name)
            }
        }

        if mail.header.cc.is_some() {
            for to in mail.header.cc.as_ref().unwrap() {
                if !EmailAddress::is_valid(to) {
                    ic_cdk::trap(format!("{} is not valid", to).as_str())
                }
    
                let email = EmailAddress::from_str(&to).unwrap();
                let domain_name = email.domain().to_string();
                selected_domains.push(domain_name)
            }
        }

        selected_domains
        
    }

    pub fn get_newsletter_subscribers(&self, newsletter_id : NEWSLETTER_ID) -> Result<Vec<EMAIL_ADDRESS>, MailError>{
        let set = self.newsletter_subscribers.get(&newsletter_id).ok_or(MailError::NotFound)?;
        Ok(set.keys().cloned().collect())
    }

    pub fn subscribe_to_newsletter(&mut self, newsletter_id : NEWSLETTER_ID, email_address : EMAIL_ADDRESS, p : Principal) -> Result<(), MailError> {
        let subscribe_set = self.newsletter_subscribers.get_mut(&newsletter_id).unwrap();
        if subscribe_set.contains_key(&email_address) {
            return Err(MailError::AddressExist);
        }
        subscribe_set.insert(email_address, p);

        Ok(())
    }

    pub fn get_newsletters(&self) -> Vec<(NEWSLETTER_ID, Newsletter)> {
        self.newsletter.clone().into_iter().collect()
    }

    pub fn unsubscribe_to_newsletter(&mut self, newsletter_id : NEWSLETTER_ID, email_address : EMAIL_ADDRESS, p : Principal) -> Result<(), MailError> {
        let subscribe_set = self.newsletter_subscribers.get_mut(&newsletter_id).unwrap();
        if subscribe_set.contains_key(&email_address) {
            return Err(MailError::AddressExist);
        }
        let b = subscribe_set.get(&email_address).unwrap().clone();

        if p == b {
            subscribe_set.remove(&email_address);
        } else {
            return Err(MailError::NotAuthorized);
        }

        Ok(())
    }

    pub fn get_newsletter(&self, mail_id : MAIL_ID) -> Result<Newsletter, MailError> {
        let n = self.newsletter.get(&mail_id).cloned().ok_or(MailError::NotFound)?;
        Ok(n)
    }


    pub fn create_newletter(&mut self, newsletter_id : NEWSLETTER_ID, letter : Newsletter) -> Result<(), MailError> {
        self.newsletter.insert(newsletter_id.clone(), letter);
        // self._newsletter_subscribers_addr.insert(newsletter_id.clone(), HashSet::new());
        self.newsletter_subscribers.insert(newsletter_id, HashMap::new());
        Ok(())
    }

    pub fn get_mail_transfer_agent_url(&self) -> String {
        return self.config.mta_url.clone();
    }
}