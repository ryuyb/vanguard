export type SpotlightItem = {
  id: string;
  cipherId: string;
  title: string;
  subtitle: string;
  searchText: string;
  type: number;
  iconUrl: string | null;
};

export type CopyField = "username" | "password" | "totp";

export type DetailAction = {
  label: string;
  shortcut: readonly string[];
  field: CopyField;
  requiresTotp?: boolean;
};
