export type VaultPageState = "loading" | "ready" | "error";

export type CipherIconLoadState = "idle" | "loading" | "loaded" | "fallback";

export type CipherTypeFilter =
  | "all"
  | "login"
  | "card"
  | "identify"
  | "note"
  | "ssh_key";

export type CipherSortBy = "title" | "created" | "modified";
export type CipherSortDirection = "asc" | "desc";

export type FolderTreeNode = {
  key: string;
  label: string;
  folderId: string | null;
  children: FolderTreeNode[];
};

export type FolderTreeNodeDraft = {
  key: string;
  label: string;
  folderId: string | null;
  childrenMap: Map<string, FolderTreeNodeDraft>;
};
