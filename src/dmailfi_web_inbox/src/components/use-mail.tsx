import { atom, useAtom } from "jotai"

import { Mail, mails } from "@/data"

type Config = {
  selected: Mail["id"] | null
}

const configAtom = atom(0)

export function useMail() {
  return useAtom(configAtom)
}