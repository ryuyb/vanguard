import type { VaultCipherItemDto, VaultFolderItemDto } from "@/bindings";
import { toErrorText } from "@/features/auth/shared/utils";
import type {
  CipherTypeFilter,
  FolderTreeNode,
  FolderTreeNodeDraft,
} from "@/features/vault/types";
import { appI18n, getLocaleCollator } from "@/i18n";

export function toTypeFilterLabel(filter: CipherTypeFilter): string {
  if (filter === "login") {
    return appI18n.t("vault.page.filters.types.login");
  }
  if (filter === "card") {
    return appI18n.t("vault.page.filters.types.card");
  }
  if (filter === "identify") {
    return appI18n.t("vault.page.filters.types.identity");
  }
  if (filter === "note") {
    return appI18n.t("vault.page.filters.types.note");
  }
  if (filter === "ssh_key") {
    return appI18n.t("vault.page.filters.types.sshKey");
  }
  return appI18n.t("vault.page.filters.types.all");
}

export function toVaultErrorText(error: unknown): string {
  return toErrorText(error, appI18n.t("vault.feedback.loadError"));
}

export function toAvatarText(email: string | null | undefined): string {
  const normalized = (email ?? "").trim();
  if (!normalized) {
    return "??";
  }
  const head = normalized.split("@")[0] ?? normalized;
  const compacted = head.replace(/[^a-zA-Z0-9]/g, "");
  return (compacted.slice(0, 2) || "??").toUpperCase();
}

export function sortFolders(
  folders: VaultFolderItemDto[],
): VaultFolderItemDto[] {
  const collator = getLocaleCollator();
  return [...folders].sort((left, right) =>
    collator.compare(left.name ?? "", right.name ?? ""),
  );
}

export function toSortableDate(value: string | null | undefined): number {
  if (!value) {
    return Number.NEGATIVE_INFINITY;
  }
  const timestamp = Date.parse(value);
  if (Number.isNaN(timestamp)) {
    return Number.NEGATIVE_INFINITY;
  }
  return timestamp;
}

export function firstNonEmptyText(
  ...values: Array<string | null | undefined>
): string | null {
  for (const value of values) {
    if (typeof value === "string" && value.trim().length > 0) {
      return value;
    }
  }
  return null;
}

function normalizeFolderSegments(folderName: string | null): string[] {
  const normalized = (folderName ?? "").trim();
  const rawSegments = normalized.split(/[\\/]+/).map((item) => item.trim());
  const segments = rawSegments.filter((item) => item.length > 0);
  if (segments.length > 0) {
    return segments;
  }
  return [appI18n.t("vault.page.folders.untitledFolder")];
}

export function buildFolderTree(
  folders: VaultFolderItemDto[],
): FolderTreeNode[] {
  const root = new Map<string, FolderTreeNodeDraft>();

  // 第一遍：收集所有实际存在的完整 folder 路径
  const existingFolderPaths = new Set<string>();
  for (const folder of folders) {
    const segments = normalizeFolderSegments(folder.name);
    existingFolderPaths.add(segments.join("/"));
  }

  // 第二遍：构建树，如果父路径不存在则使用原始 name
  for (const folder of folders) {
    const segments = normalizeFolderSegments(folder.name);

    // 检查是否所有父路径都存在（作为独立的 folder）
    let canBuildHierarchy = true;
    const pathParts: string[] = [];
    for (let index = 0; index < segments.length - 1; index += 1) {
      pathParts.push(segments[index]);
      const parentPath = pathParts.join("/");
      if (!existingFolderPaths.has(parentPath)) {
        canBuildHierarchy = false;
        break;
      }
    }

    if (!canBuildHierarchy) {
      // 父路径不存在，直接使用原始 name 作为根节点
      const originalName =
        folder.name ?? appI18n.t("vault.page.folders.untitledFolder");
      const key = folder.id;
      root.set(key, {
        key,
        label: originalName,
        folderId: folder.id,
        childrenMap: new Map<string, FolderTreeNodeDraft>(),
      });
      continue;
    }

    // 父路径存在，正常构建层级结构
    let current = root;
    pathParts.length = 0;

    for (let index = 0; index < segments.length; index += 1) {
      const segment = segments[index];
      pathParts.push(segment);
      const key = pathParts.join("/");

      let node = current.get(segment);
      if (!node) {
        node = {
          key,
          label: segment,
          folderId: null,
          childrenMap: new Map<string, FolderTreeNodeDraft>(),
        };
        current.set(segment, node);
      }

      if (index === segments.length - 1) {
        node.folderId = folder.id;
      }

      current = node.childrenMap;
    }
  }

  const toSortedNodes = (
    map: Map<string, FolderTreeNodeDraft>,
  ): FolderTreeNode[] => {
    const collator = getLocaleCollator();
    return [...map.values()]
      .sort((left, right) => collator.compare(left.label, right.label))
      .map((node) => ({
        key: node.key,
        label: node.label,
        folderId: node.folderId,
        children: toSortedNodes(node.childrenMap),
      }));
  };

  return toSortedNodes(root);
}

export function collectFolderTreeKeys(nodes: FolderTreeNode[]): string[] {
  const keys: string[] = [];

  const walk = (items: FolderTreeNode[]) => {
    for (const item of items) {
      keys.push(item.key);
      walk(item.children);
    }
  };

  walk(nodes);
  return keys;
}

export function countNodeCiphers(
  node: FolderTreeNode,
  folderCipherCount: Map<string, number>,
): number {
  const selfCount = node.folderId
    ? (folderCipherCount.get(node.folderId) ?? 0)
    : 0;
  let childCount = 0;
  for (const child of node.children) {
    childCount += countNodeCiphers(child, folderCipherCount);
  }
  return selfCount + childCount;
}

export function getCipherPrimaryUri(
  cipher: Pick<VaultCipherItemDto, "name"> & {
    uri?: string | null;
    uris?: Array<{ uri: string | null } | string | null> | null;
    login?: {
      uri?: string | null;
      uris?: Array<{ uri: string | null }> | null;
    } | null;
    data?: {
      uri?: string | null;
      uris?: Array<{ uri: string | null }> | null;
    } | null;
  },
): string | null {
  const itemUris = (cipher.uris ?? []).map((entry) => {
    if (typeof entry === "string") {
      return entry;
    }
    return entry?.uri;
  });

  const candidates = [
    cipher.uri,
    ...itemUris,
    cipher.login?.uri,
    ...(cipher.login?.uris ?? []).map((item) => item.uri),
    cipher.data?.uri,
    ...(cipher.data?.uris ?? []).map((item) => item.uri),
  ];

  return firstNonEmptyText(...candidates);
}

/**
 * Extracts the hostname from a URI for icon lookup.
 * Returns null for invalid hosts (localhost, IPs, etc.)
 */
export function toCipherIconHost(
  uri: string | null | undefined,
): string | null {
  const normalizedUri = (uri ?? "").trim();
  if (!normalizedUri) {
    return null;
  }

  try {
    const parsed = new URL(normalizedUri);
    return normalizeIconHost(parsed.hostname);
  } catch {
    try {
      const parsed = new URL(`https://${normalizedUri}`);
      return normalizeIconHost(parsed.hostname);
    } catch {
      return null;
    }
  }
}

function normalizeIconHost(hostname: string): string | null {
  const normalized = hostname.trim().toLowerCase();
  if (!normalized) {
    return null;
  }
  if (normalized === "localhost") {
    return null;
  }
  if (/^\d+\.\d+\.\d+\.\d+$/.test(normalized)) {
    return null;
  }
  if (!/^[a-z0-9.-]+$/.test(normalized)) {
    return null;
  }
  if (normalized.startsWith(".") || normalized.includes("..")) {
    return null;
  }
  return normalized;
}
