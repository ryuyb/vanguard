import { Fragment, type PropsWithChildren } from "react";

export function AppLocaleProvider({ children }: PropsWithChildren) {
  return <Fragment>{children}</Fragment>;
}
