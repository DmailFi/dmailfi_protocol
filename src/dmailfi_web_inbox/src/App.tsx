import { useEffect, useState } from "react";
import { Mail } from "@/Mail";
import { accounts, mails as mock_mails } from "@/data";
import { ThemeProvider } from "@/theme";
import {
  MailRegistry,
  MailStorage,
  MAINNET_DMAILFI_REGISTRY_CANISTER_ID,
} from "@dmailfi/sdk";
import { AuthClient } from "@dfinity/auth-client";
import { Toaster } from "@/components/ui/sonner";
import { toast } from "sonner";
import Editor from "./components/editor";
import { useSetAtom, useAtom } from "jotai";
import { storageAtom } from "@/store/app";

function App() {
  let [mails, setMails] = useState([]);
  let [loadingMail, setLoadingMails] = useState(true);
  //@ts-ignore
  let setStorageAtom = useSetAtom(storageAtom)

  useEffect(() => {
    let task = async () => {
      let auth = await AuthClient.create();
      let is_auth = await auth.isAuthenticated();
      if (is_auth) {
        let identity = auth.getIdentity();
        let registry = new MailRegistry(
          MAINNET_DMAILFI_REGISTRY_CANISTER_ID,
          identity
        );
        let canister_rslt = await registry.lookup_user();
        if ("Err" in canister_rslt) {
          toast.error("There's error trying to lookup user, check the console");
          console.log(canister_rslt.Err);
          return;
        }

        let id = canister_rslt.Ok;
        let mail_storage = new MailStorage(id, identity);
        setStorageAtom(mail_storage)
        let mail_response = await mail_storage.get_mails();

        if ("Err" in mail_response) {
          toast.error("There's error Fetching Mails, check the console");
          console.log(mail_response.Err);
          return;
        }


        let mails = mail_response.Ok.map((inbox) => {
          return {
            id: inbox.mail_id,
            name: inbox.header.sender_name[0] || inbox.header.from,
            email: inbox.header.from,
            subject: inbox.header.subject,
            text: inbox.content[0],
            date: inbox.header.timestamp,
            read: inbox.read,
            content_type: inbox.header.content_type[0],
            labels: [],
          }
        });

        setMails(mails)
      } else {
      }
    };

    // task()
  });
  return (
    <>
      <Toaster />
      <ThemeProvider defaultTheme="system" storageKey="vite-ui-theme">
        <div className="hidden flex-col md:flex">
          <Mail
            accounts={accounts}
            mails={mock_mails}
            defaultLayout={undefined}
            defaultCollapsed={undefined}
            navCollapsedSize={4}
          />
        </div>

        {/* <div className="fixed bottom-0 right-0 w-screen h-screen lg:h-auto lg:w-[45vw] lg:mb-3 lg:mr-8">
          <Editor/>
        </div> */}
      </ThemeProvider>
    </>
  );
}

export default App;
