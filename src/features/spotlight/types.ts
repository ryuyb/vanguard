export type SpotlightItem = {
  id: string;
  cipherId: string;
  title: string;
  subtitle: string;
  searchText: string;
  type: number;
  iconHostname: string | null;
  iconData?: string | null;
};

export type CopyField =
  | "username"
  | "password"
  | "totp"
  | "notes"
  | { customField: { index: number } }
  | { uri: { index: number } }
  | "cardNumber"
  | "cardCode"
  | "email"
  | "phone"
  | "sshPrivateKey"
  | "sshPublicKey";

export type DetailAction = {
  label: string;
  shortcut: readonly string[];
  field: CopyField;
  requiresTotp?: boolean;
};
