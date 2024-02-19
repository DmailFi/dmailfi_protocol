use candid::candid_method;
use ic_cdk::query;
#[ic_cdk::query]
fn greet(name: String) -> String {
    format!("Hello, {}!", name)
}

#[query]
#[candid_method(query)]
fn export_candid() -> String {
    ic_cdk::export_candid!();
    __export_service()
}
