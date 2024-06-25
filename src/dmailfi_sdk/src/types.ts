import { Identity } from "@dfinity/agent";
import {createActor as createStorageActor} from "./declarations/dmailfi_core"
import {createActor as createRegistryActor} from "./declarations/dmailfi_icp_backend"
import {Result as LookUpUserResponse} from "./declarations/dmailfi_icp_backend/dmailfi_icp_backend.did"

export class MailStorage {
    private backend_actor
    constructor(canister_id : string, identity ?: Identity) {
        this.backend_actor = createStorageActor(canister_id, {
            agentOptions: {
                identity
            }
        })
    }


    get_mails(page ?: number) {
        return this.backend_actor.get_mails(page ? [BigInt(page)] : [])
    }

    fetch_mail(id : string) {
        return this.backend_actor.get_mail(id)
    }

    get_actor() {
        return this.backend_actor
    }

}

export class MailRegistry {
    private backend_actor
    constructor(canister_id : string, identity ?: Identity) {
        this.backend_actor = createRegistryActor(canister_id, {
            agentOptions: {
                identity
            }
        })
    }

    async lookup_user() : Promise<LookUpUserResponse> {
        return this.backend_actor.lookup_user()
    }

    get_actor() {
        return this.backend_actor
    }
    
}