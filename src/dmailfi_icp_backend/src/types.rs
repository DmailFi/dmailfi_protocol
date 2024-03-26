use std::collections::{HashMap, HashSet};

use candid::{CandidType, Deserialize, Principal};

pub type DOMAIN_NAME = String;
pub type CANISTER_ID = Principal;
#[derive(Default)]
pub struct Ledger {
    domains : HashMap<DOMAIN_NAME, String>,
    custodians: HashSet<String>,
    pending_canister: HashMap<CANISTER_ID, Principal>
}

#[derive(CandidType, Deserialize)]
pub enum RegistryError {
    NotFound,
    FailedToUpgrade(String),
    FailedToCreateCanister,
    FailedToInstallCode(String)
}

impl Ledger {
    pub fn lookup_domain_name(&self, domain_name : DOMAIN_NAME) -> Result<String, RegistryError> {
        let option = self.domains.get(&domain_name).cloned();
        if option.is_none() {
            Err(RegistryError::NotFound)
        } else {
            Ok(option.unwrap())
        }
    }

    pub fn is_custodian(&self,principal_id : String) -> Result<(), String> {
        if self.custodians.contains(&principal_id) {
            Ok(())
        } else {
            Err("You are not custodian".to_string())
        }
    }

    pub fn add_to_pending_canister(&mut self, canister_id: CANISTER_ID, principal : Principal) {
        self.pending_canister.insert(canister_id, principal);
    }

    pub fn add_domain(&mut self, domain_name : DOMAIN_NAME, principal_str : String) {
        self.domains.insert(domain_name, principal_str);
    }

    pub fn get_all_domain_canisters(&self) -> Vec<String> {
        self.domains.values().cloned().collect()
    }
    pub fn get_domain_details(&self, domain_name : DOMAIN_NAME) -> Result<String, RegistryError> {
        match self.domains.get(&domain_name){
            Some(details) => {
                let details_str = format!(
                    "Domain: {} is managed by canister: {}",
                    domain_name,
                    details
                );
                Ok(details_str)
            }
            None => Err(RegistryError::NotFound)
        }
    }
}