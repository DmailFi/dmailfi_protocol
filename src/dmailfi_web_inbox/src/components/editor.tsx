
import {
  PaperClipIcon,
  XMarkIcon
} from "@heroicons/react/20/solid";
import { AuthClient } from "@dfinity/auth-client";
import { Identity } from "@dfinity/agent/lib/cjs/auth";
import { string } from "yup";
import { useState } from "react";
import { MailStorage } from "@dmailfi/sdk";
import { useAtomValue } from "jotai";
import { storageAtom } from "@/store/app";
import { toast } from "sonner";
import { Mail } from "@dmailfi/sdk/dist/declarations/dmailfi_core/dmailfi_core.did";

function isEmail(email: string) {
    return string().email().isValidSync(email);
  }
  
  function AdditionalContent(props: any) {
    return (
      <>
        <div className="mb-4 mt-2 text-sm text-primary-700 dark:text-primary-800">
          {props.info}
        </div>
        <div className="flex">
          <button
            onClick={props.action}
            type="button"
            className="mr-2 inline-flex items-center rounded-lg bg-primary-600 px-3 py-1.5 text-center text-xs font-medium text-white hover:bg-primary-800 focus:ring-4 focus:ring-primary-300 dark:bg-primary-800 dark:hover:bg-primary-900"
          >
            {props.action_name}
          </button>
          <button
            onClick={props.onDismiss}
            type="button"
            className="rounded-lg border border-primary-700 bg-transparent px-3 py-1.5 text-center text-xs font-medium text-primary-700 hover:bg-primary-800 hover:text-white focus:ring-4 focus:ring-primary-300 dark:border-primary-800 dark:text-primary-800 dark:hover:text-white"
          >
            Dismiss
          </button>
        </div>
      </>
    );
  }
  
  export default function Editor() {
    let [files, setFiles] = useState([]);
    let [message, setMessage] = useState();
    let [receipients, setReceipient] = useState([]);
    let [userIdenty, setIdentity] = useState<Identity>();
    let [sending, setSending] = useState(false)
    let [profile, setProfile] = useState(null)
    let [subject, setSubject] = useState(null)

    //@ts-ignore
    let mail_storage : MailStorage | null = useAtomValue(storageAtom)

    const check_email_and_update = async (
        email: any,
        idx: number,
      ) => {
        console.log("Starting to check address book");

        // console.log(`Output: ${principal}`);
        // if (principal) {
        //   receipients[idx].e2e = principal;
        //   setReceipient([...receipients]);
        // } else {
        //   receipients[idx].e2e = false;
        //   setReceipient([...receipients]);
        // }
      };

    return (
      <>
        <div className="w-full h-full p-4 rounded-md shadow-xl bg-white text-gray-900 flex justify-center items-center flex-col">
          <div className=" w-full">
            <div className="flex justify-end">
                <button>
                    <XMarkIcon className="w-6 h-6"/>
                </button>
            </div>
            <div className="flex space-x-1 flex-wrap p-2 rounded-lg border-indigo-500 border-solid">
              {receipients.map((r, i) => {
                if (r.e2e) {
                  return (
                    <span
                      key={i}
                      className="inline-flex items-center rounded-full bg-indigo-50 px-2 py-1 text-base font-medium text-indigo-700 ring-1 ring-inset ring-pink-700/10"
                    >
                      {r.email}
                      <button
                        onClick={(e) => {
                          
                        }}
                      >
                        <svg
                          xmlns="http://www.w3.org/2000/svg"
                          fill="none"
                          viewBox="0 0 24 24"
                          stroke-width="1.5"
                          stroke="currentColor"
                          className="w-6 h-6"
                        >
                          <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            d="M6 18L18 6M6 6l12 12"
                          />
                        </svg>
                      </button>
                    </span>
                  );
                } else if (r.e2e === false) {
                  return (
                    <span
                      key={i}
                      className="inline-flex items-center rounded-full bg-green-50 px-2 py-1 text-base font-medium text-green-700 ring-1 ring-inset ring-pink-700/10"
                    >
                      {r.email}
                      <button
                        onClick={(e) => {
                          
                        }}
                      >
                        <svg
                          xmlns="http://www.w3.org/2000/svg"
                          fill="none"
                          viewBox="0 0 24 24"
                          stroke-width="1.5"
                          stroke="currentColor"
                          className="w-6 h-6"
                        >
                          <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            d="M6 18L18 6M6 6l12 12"
                          />
                        </svg>
                      </button>
                    </span>
                  );
                } else if (r.e2e === null) {
                  return (
                    <span
                      key={i}
                      className="inline-flex items-center rounded-full bg-gray-50 px-2 py-1 text-base font-medium text-gray-700 ring-1 ring-inset ring-pink-700/10"
                    >
                      {r.email}
                      <button
                        onClick={(e) => {
                          
                        }}
                      >
                        <svg
                          xmlns="http://www.w3.org/2000/svg"
                          fill="none"
                          viewBox="0 0 24 24"
                          stroke-width="1.5"
                          stroke="currentColor"
                          className="w-6 h-6"
                        >
                          <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            d="M6 18L18 6M6 6l12 12"
                          />
                        </svg>
                      </button>
                    </span>
                  );
                }
              })}
  
              <input
                onKeyUp={(e) => {
                    let task = () => {
                        //@ts-ignore
                        let email = e.target.value;
                        if (!isEmail(email)) {
                          return;
                        }
                        console.log(email);
                        let n_idx = receipients.length;

                        check_email_and_update(
                            email,
                            n_idx,
                          ).then((e) => {
                            //
                          });

                        receipients.push({
                          email,
                          e2e: null,
                        });
                        //@ts-ignore
                        e.target.value = null;
                        setReceipient([...receipients]);
                      };
                      if (e.key === "Enter") {
                        task();
                      } else if (e.key === " ") {
                        task();
                      }
                }}
                placeholder="Type Email and press enter"
                className="min-w-[33%] form-input focus:outline-none focus:ring-0 p-0 border-0 font-light text-xl"
              />
            </div>
            <div className="flex space-x-1 flex-wrap p-2 rounded-lg border-indigo-500 border-solid">
              <input
                onChange={e => {
                    setSubject(e.target.value)
                }}
                placeholder="Type Subject"
                className="min-w-[33%] form-input focus:outline-none focus:ring-0 p-0 border-0 font-light text-xl"
              />
            </div>
  
            <form
              onSubmit={(e) => {
                e.preventDefault();
              }}
              className="relative"
            >
              <div className="overflow-hidden rounded-lg border border-gray-300 shadow-sm focus-within:border-indigo-500 focus-within:ring-1 focus-within:ring-indigo-500">
                <label htmlFor="description" className="sr-only">
                  Description
                </label>
                <textarea
                  rows={12}
                  onChange={(e) => {
                    //@ts-ignore
                    setMessage(e.target.value);
                  }}
                  name="description"
                  id="description"
                  className="block w-full border-0 p-2.5 text-xl font-light placeholder:text-gray-400 focus:ring-0"
                  placeholder="Write your message"
                  defaultValue={""}
                />
  
                {/* Spacer element to match the height of the toolbar */}
                <div aria-hidden="true">
                  <div className="py-2">
                    <div className="h-9" />
                  </div>
                  <div className="h-px" />
                  <div className="py-2">
                    <div className="py-px">
                      <div className="h-9" />
                    </div>
                  </div>
                </div>
              </div>
  
              <div className="absolute inset-x-px bottom-0 p-1">
                {files.map((v, i) => (
                  <span
                    key={i}
                    className="inline-flex items-center rounded-full bg-indigo-50 px-2 py-1 text-xs font-medium text-indigo-700 ring-1 ring-inset ring-pink-700/10"
                  >
                    <svg
                      xmlns="http://www.w3.org/2000/svg"
                      fill="none"
                      viewBox="0 0 24 24"
                      strokeWidth={1.5}
                      stroke="currentColor"
                      className="w-5 h-5"
                    >
                      <path
                        strokeLinecap="round"
                        strokeLinejoin="round"
                        d="M19.5 14.25v-2.625a3.375 3.375 0 00-3.375-3.375h-1.5A1.125 1.125 0 0113.5 7.125v-1.5a3.375 3.375 0 00-3.375-3.375H8.25m0 12.75h7.5m-7.5 3H12M10.5 2.25H5.625c-.621 0-1.125.504-1.125 1.125v17.25c0 .621.504 1.125 1.125 1.125h12.75c.621 0 1.125-.504 1.125-1.125V11.25a9 9 0 00-9-9z"
                      />
                    </svg>
  
                    {v.name}
                    <button
                      onClick={(e) => {
                        
                      }}
                    >
                      <svg
                        xmlns="http://www.w3.org/2000/svg"
                        fill="none"
                        viewBox="0 0 24 24"
                        strokeWidth="1.5"
                        stroke="currentColor"
                        className="w-6 h-6"
                      >
                        <path
                          strokeLinecap="round"
                          strokeLinejoin="round"
                          d="M6 18L18 6M6 6l12 12"
                        />
                      </svg>
                    </button>
                  </span>
                ))}
  
                {/* Actions: These are ju
          
          st examples to demonstrate the concept, replace/wire these up however makes sense for your project. */}
  
                <div className="flex items-center justify-between space-x-3 border-t border-gray-200 px-2 py-2 sm:px-3">
                  <div className="flex">
                    <input
                      onChange={(e) => {
                        //@ts-ignore
                        setFiles([...files, ...e.target.files]);
                      }}
                      id="fileInput"
                      type="file"
                      hidden
                    />
                    <button
                      onClick={(e) => {
                        let node = document.getElementById("fileInput");
                        node.click();
                      }}
                      type="button"
                      className="group -my-2 -ml-2 inline-flex items-center rounded-full px-3 py-2 text-left text-gray-400"
                    >
                      <PaperClipIcon
                        className="-ml-1 mr-2 h-5 w-5 group-hover:text-gray-500"
                        aria-hidden="true"
                      />
                      <span className="text-sm italic text-gray-500 group-hover:text-gray-600">
                        Attach a file
                      </span>
                    </button>
                  </div>
                  <div className="flex-shrink-0">
                    {sending ? (
                      <button
                      disabled
                      type="button"
                      className="text-white bg-blue-700 hover:bg-blue-800 focus:ring-4 focus:outline-none focus:ring-blue-300 font-medium rounded-lg text-sm px-5 py-2.5 text-center me-2 dark:bg-blue-600 dark:hover:bg-blue-700 dark:focus:ring-blue-800 inline-flex items-center"
                    >
                      <svg
                        aria-hidden="true"
                        role="status"
                        className="inline w-4 h-4 me-3 text-white animate-spin"
                        viewBox="0 0 100 101"
                        fill="none"
                        xmlns="http://www.w3.org/2000/svg"
                      >
                        <path
                          d="M100 50.5908C100 78.2051 77.6142 100.591 50 100.591C22.3858 100.591 0 78.2051 0 50.5908C0 22.9766 22.3858 0.59082 50 0.59082C77.6142 0.59082 100 22.9766 100 50.5908ZM9.08144 50.5908C9.08144 73.1895 27.4013 91.5094 50 91.5094C72.5987 91.5094 90.9186 73.1895 90.9186 50.5908C90.9186 27.9921 72.5987 9.67226 50 9.67226C27.4013 9.67226 9.08144 27.9921 9.08144 50.5908Z"
                          fill="#E5E7EB"
                        />
                        <path
                          d="M93.9676 39.0409C96.393 38.4038 97.8624 35.9116 97.0079 33.5539C95.2932 28.8227 92.871 24.3692 89.8167 20.348C85.8452 15.1192 80.8826 10.7238 75.2124 7.41289C69.5422 4.10194 63.2754 1.94025 56.7698 1.05124C51.7666 0.367541 46.6976 0.446843 41.7345 1.27873C39.2613 1.69328 37.813 4.19778 38.4501 6.62326C39.0873 9.04874 41.5694 10.4717 44.0505 10.1071C47.8511 9.54855 51.7191 9.52689 55.5402 10.0491C60.8642 10.7766 65.9928 12.5457 70.6331 15.2552C75.2735 17.9648 79.3347 21.5619 82.5849 25.841C84.9175 28.9121 86.7997 32.2913 88.1811 35.8758C89.083 38.2158 91.5421 39.6781 93.9676 39.0409Z"
                          fill="currentColor"
                        />
                      </svg>
                      Sending....
                    </button>
                    ) : (<button
                    onClick={() => {
                        if(!mail_storage) {
                            toast.error("You not connected to a any personal mail storage")
                        }
                        let encoder = new TextEncoder()
                        let body = encoder.encode(message)
                        let actor = mail_storage.get_actor()
                        let mail : Mail = {
                            header: {
                                from: "",
                                to: receipients.map(r => r.email),
                                content_type: ["text/plain"],
                                subject: [subject],
                                cc: [],
                                bcc: [],
                                sender_name: [],
                                timestamp: BigInt(0),
                                sender_canister_id: [],
                                sender_channel: []
                            },
                            body
                        }
                        actor.send_mail(mail).then(result => {
                            if("Err" in result) {
                                let err = result.Err
                                toast.error(JSON.stringify(err))
                            }

                            if("Ok" in result) {
                                toast.success("Message Sent!!")
                            }
                        })
                    }}
                      type="button"
                      className="text-white bg-blue-700 hover:bg-blue-800 focus:ring-4 focus:outline-none focus:ring-blue-300 font-medium rounded-lg text-sm px-5 py-2.5 text-center me-2 dark:bg-blue-600 dark:hover:bg-blue-700 dark:focus:ring-blue-800 inline-flex items-center"
                    >
                     
                      Send Message
                    </button>)}
                    
                  </div>
                </div>
              </div>
            </form>
            {/* <TextField /> */}
          </div>
        </div>
      </>
    );
  }
  