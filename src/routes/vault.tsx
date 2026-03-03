import { createFileRoute, redirect, useNavigate } from "@tanstack/react-router";
import {
  Archive,
  ArrowUpDown,
  ChevronDown,
  ChevronRight,
  LoaderCircle,
  Lock,
  LogOut,
  Search,
  Star,
  Trash2,
  UserRound,
  X,
} from "lucide-react";
import { AnimatePresence, motion } from "motion/react";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import {
  commands,
  type VaultCipherDetailDto,
  type VaultCipherItemDto,
  type VaultFolderItemDto,
  type VaultViewDataResponseDto,
} from "@/bindings";
import { Button } from "@/components/ui/button";
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/components/ui/collapsible";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuRadioGroup,
  DropdownMenuRadioItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Input } from "@/components/ui/input";
import { ScrollArea } from "@/components/ui/scroll-area";
import { resolveSessionRoute } from "@/lib/route-session";

export const Route = createFileRoute("/vault")({
  beforeLoad: async () => {
    const target = await resolveSessionRoute();
    if (target !== "/vault") {
      throw redirect({ to: target });
    }
  },
  component: VaultPage,
});

const ALL_ITEMS_ID = "__all_items__";
const FAVORITES_ID = "__favorites__";
const TRASH_ID = "__trash__";

type VaultPageState = "loading" | "needsLogin" | "locked" | "ready" | "error";
type CipherTypeFilter =
  | "all"
  | "login"
  | "card"
  | "identify"
  | "note"
  | "ssh_key";
type CipherSortBy = "title" | "created" | "modified";
type CipherSortDirection = "asc" | "desc";

function toTypeFilterLabel(filter: CipherTypeFilter) {
  if (filter === "login") {
    return "Login";
  }
  if (filter === "card") {
    return "Card";
  }
  if (filter === "identify") {
    return "Identify";
  }
  if (filter === "note") {
    return "Note";
  }
  if (filter === "ssh_key") {
    return "SSH key";
  }
  return "All types";
}

function errorToText(error: unknown) {
  if (typeof error === "string") {
    return error;
  }
  if (error instanceof Error) {
    return error.message;
  }
  return "加载 vault 数据失败，请稍后重试。";
}

function toAvatarText(email: string | null | undefined) {
  const normalized = (email ?? "").trim();
  if (!normalized) {
    return "??";
  }
  const head = normalized.split("@")[0] ?? normalized;
  const compacted = head.replace(/[^a-zA-Z0-9]/g, "");
  return (compacted.slice(0, 2) || "??").toUpperCase();
}

function toCipherTypeLabel(type: number | null) {
  if (type === 1) {
    return "Login";
  }
  if (type === 2) {
    return "Secure Note";
  }
  if (type === 3) {
    return "Card";
  }
  if (type === 4) {
    return "Identity";
  }
  if (type === 5) {
    return "SSH Key";
  }
  return "Unknown";
}

function formatRevisionDate(value: string | null) {
  if (!value) {
    return "Unknown";
  }
  const parsed = new Date(value);
  if (Number.isNaN(parsed.getTime())) {
    return value;
  }
  return parsed.toLocaleString();
}

function sortFolders(folders: VaultFolderItemDto[]) {
  return [...folders].sort((left, right) =>
    (left.name ?? "").localeCompare(right.name ?? "", "zh-Hans-CN"),
  );
}

function toSortableDate(value: string | null | undefined) {
  if (!value) {
    return Number.NEGATIVE_INFINITY;
  }
  const timestamp = Date.parse(value);
  if (Number.isNaN(timestamp)) {
    return Number.NEGATIVE_INFINITY;
  }
  return timestamp;
}

function firstNonEmptyText(
  ...values: Array<string | null | undefined>
): string | null {
  for (const value of values) {
    if (typeof value === "string" && value.trim().length > 0) {
      return value;
    }
  }
  return null;
}

type FolderTreeNode = {
  key: string;
  label: string;
  folderId: string | null;
  children: FolderTreeNode[];
};

type FolderTreeNodeDraft = {
  key: string;
  label: string;
  folderId: string | null;
  childrenMap: Map<string, FolderTreeNodeDraft>;
};

function normalizeFolderSegments(folderName: string | null) {
  const normalized = (folderName ?? "").trim();
  const rawSegments = normalized.split(/[\\/]+/).map((item) => item.trim());
  const segments = rawSegments.filter((item) => item.length > 0);
  if (segments.length > 0) {
    return segments;
  }
  return ["Untitled folder"];
}

function buildFolderTree(folders: VaultFolderItemDto[]) {
  const root = new Map<string, FolderTreeNodeDraft>();

  for (const folder of folders) {
    const segments = normalizeFolderSegments(folder.name);
    let current = root;
    const pathParts: string[] = [];

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
    return [...map.values()]
      .sort((left, right) =>
        left.label.localeCompare(right.label, "zh-Hans-CN"),
      )
      .map((node) => ({
        key: node.key,
        label: node.label,
        folderId: node.folderId,
        children: toSortedNodes(node.childrenMap),
      }));
  };

  return toSortedNodes(root);
}

function collectFolderTreeKeys(nodes: FolderTreeNode[]) {
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

function countNodeCiphers(
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

function VaultPage() {
  const navigate = useNavigate({ from: "/vault" });
  const searchInputRef = useRef<HTMLInputElement | null>(null);
  const inlineSearchInputRef = useRef<HTMLInputElement | null>(null);
  const [headerSearchQuery, setHeaderSearchQuery] = useState("");
  const [cipherSearchQuery, setCipherSearchQuery] = useState("");
  const [isInlineSearchOpen, setIsInlineSearchOpen] = useState(false);
  const [typeFilter, setTypeFilter] = useState<CipherTypeFilter>("all");
  const [sortBy, setSortBy] = useState<CipherSortBy>("modified");
  const [sortDirection, setSortDirection] =
    useState<CipherSortDirection>("desc");
  const [userEmail, setUserEmail] = useState("未登录");
  const [userBaseUrl, setUserBaseUrl] = useState("未知服务");
  const [isRefreshing, setIsRefreshing] = useState(false);
  const [isLocking, setIsLocking] = useState(false);
  const [isLoggingOut, setIsLoggingOut] = useState(false);
  const [pageState, setPageState] = useState<VaultPageState>("loading");
  const [errorText, setErrorText] = useState("");
  const [viewData, setViewData] = useState<VaultViewDataResponseDto | null>(
    null,
  );
  const [selectedMenuId, setSelectedMenuId] = useState(ALL_ITEMS_ID);
  const [selectedCipherId, setSelectedCipherId] = useState<string | null>(null);
  const [selectedCipherDetail, setSelectedCipherDetail] =
    useState<VaultCipherDetailDto | null>(null);
  const [isCipherDetailLoading, setIsCipherDetailLoading] = useState(false);
  const [cipherDetailError, setCipherDetailError] = useState("");
  const [expandedNodeKeys, setExpandedNodeKeys] = useState<Set<string>>(
    new Set<string>(),
  );
  const detailRequestSeqRef = useRef(0);

  const loadVaultData = useCallback(async () => {
    setIsRefreshing(true);
    setErrorText("");
    setPageState("loading");

    try {
      const restore = await commands.authRestoreState({});
      if (restore.status === "error") {
        setPageState("error");
        setErrorText(errorToText(restore.error));
        setUserEmail("未登录");
        setUserBaseUrl("未知服务");
        return;
      }

      setUserEmail(restore.data.email ?? "未登录");
      setUserBaseUrl(restore.data.baseUrl ?? "未知服务");
      if (restore.data.status === "needsLogin") {
        setPageState("needsLogin");
        setViewData(null);
        return;
      }

      const unlockResult = await commands.vaultIsUnlocked();
      if (unlockResult.status === "error" || !unlockResult.data) {
        setPageState("locked");
        setViewData(null);
        return;
      }

      const result = await commands.vaultGetViewData({
        page: 1,
        pageSize: 500,
      });

      if (result.status === "error") {
        const text = errorToText(result.error);
        if (text.toLowerCase().includes("vault is locked")) {
          setPageState("locked");
        } else {
          setPageState("error");
          setErrorText(text);
        }
        setViewData(null);
        return;
      }

      setViewData(result.data);
      setPageState("ready");
    } catch (error) {
      setPageState("error");
      setErrorText(errorToText(error));
      setViewData(null);
    } finally {
      setIsRefreshing(false);
    }
  }, []);

  useEffect(() => {
    void loadVaultData();
  }, [loadVaultData]);

  const onLock = async () => {
    setIsLocking(true);
    try {
      const result = await commands.vaultLock({});
      if (result.status === "ok") {
        await navigate({ to: "/unlock" });
      }
    } finally {
      setIsLocking(false);
    }
  };

  const onLogout = async () => {
    setIsLoggingOut(true);
    try {
      const result = await commands.authLogout({});
      if (result.status === "ok") {
        await navigate({ to: "/" });
      }
    } finally {
      setIsLoggingOut(false);
    }
  };

  const sortedFolders = useMemo(
    () => sortFolders(viewData?.folders ?? []),
    [viewData?.folders],
  );
  const folderTree = useMemo(
    () => buildFolderTree(sortedFolders),
    [sortedFolders],
  );
  const folderTreeKeys = useMemo(
    () => collectFolderTreeKeys(folderTree),
    [folderTree],
  );
  const folderIdSet = useMemo(
    () => new Set(sortedFolders.map((folder) => folder.id)),
    [sortedFolders],
  );

  useEffect(() => {
    setExpandedNodeKeys((previous) => {
      const validKeys = new Set(folderTreeKeys);
      const next = new Set<string>();
      for (const key of previous) {
        if (validKeys.has(key)) {
          next.add(key);
        }
      }
      if (next.size === 0) {
        for (const node of folderTree) {
          next.add(node.key);
        }
      }
      return next;
    });
  }, [folderTree, folderTreeKeys]);

  useEffect(() => {
    if (
      selectedMenuId === ALL_ITEMS_ID ||
      selectedMenuId === FAVORITES_ID ||
      selectedMenuId === TRASH_ID
    ) {
      return;
    }
    const exists = folderIdSet.has(selectedMenuId);
    if (!exists) {
      setSelectedMenuId(ALL_ITEMS_ID);
    }
  }, [folderIdSet, selectedMenuId]);

  useEffect(() => {
    if (!isInlineSearchOpen) {
      return;
    }
    const frameId = requestAnimationFrame(() => {
      inlineSearchInputRef.current?.focus();
    });
    return () => {
      cancelAnimationFrame(frameId);
    };
  }, [isInlineSearchOpen]);

  const normalizedCipherSearchQuery = cipherSearchQuery.trim().toLowerCase();

  const filteredCiphers = useMemo(() => {
    const allCiphers = viewData?.ciphers ?? [];
    const folderFiltered =
      selectedMenuId === ALL_ITEMS_ID ||
      selectedMenuId === FAVORITES_ID ||
      selectedMenuId === TRASH_ID
        ? allCiphers
        : allCiphers.filter((cipher) => cipher.folderId === selectedMenuId);

    const typeFiltered = folderFiltered.filter((cipher) => {
      if (typeFilter === "all") {
        return true;
      }
      if (typeFilter === "login") {
        return cipher.type === 1;
      }
      if (typeFilter === "note") {
        return cipher.type === 2;
      }
      if (typeFilter === "card") {
        return cipher.type === 3;
      }
      if (typeFilter === "identify") {
        return cipher.type === 4;
      }
      if (typeFilter === "ssh_key") {
        return cipher.type === 5;
      }
      return true;
    });

    const searchFiltered = !normalizedCipherSearchQuery
      ? typeFiltered
      : typeFiltered.filter((cipher) => {
          const searchText = [cipher.name, cipher.id, cipher.organizationId]
            .filter(Boolean)
            .join(" ")
            .toLowerCase();
          return searchText.includes(normalizedCipherSearchQuery);
        });

    return [...searchFiltered].sort((left, right) => {
      if (sortBy === "title") {
        const titleCompare = (left.name ?? "").localeCompare(
          right.name ?? "",
          "zh-Hans-CN",
        );
        return sortDirection === "asc" ? titleCompare : -titleCompare;
      }
      if (sortBy === "created") {
        const createdCompare =
          toSortableDate(left.creationDate) -
          toSortableDate(right.creationDate);
        return sortDirection === "asc" ? createdCompare : -createdCompare;
      }
      const modifiedCompare =
        toSortableDate(left.revisionDate) - toSortableDate(right.revisionDate);
      return sortDirection === "asc" ? modifiedCompare : -modifiedCompare;
    });
  }, [
    normalizedCipherSearchQuery,
    selectedMenuId,
    sortBy,
    sortDirection,
    typeFilter,
    viewData?.ciphers,
  ]);

  useEffect(() => {
    if (!selectedCipherId) {
      return;
    }
    const existsInList = filteredCiphers.some(
      (cipher) => cipher.id === selectedCipherId,
    );
    if (existsInList) {
      return;
    }
    detailRequestSeqRef.current += 1;
    setSelectedCipherId(null);
    setSelectedCipherDetail(null);
    setCipherDetailError("");
    setIsCipherDetailLoading(false);
  }, [filteredCiphers, selectedCipherId]);

  const loadCipherDetail = useCallback(async (cipherId: string) => {
    const normalizedCipherId = cipherId.trim();
    if (!normalizedCipherId) {
      return;
    }

    setSelectedCipherId(normalizedCipherId);
    setSelectedCipherDetail(null);
    setCipherDetailError("");
    setIsCipherDetailLoading(true);

    const requestSeq = detailRequestSeqRef.current + 1;
    detailRequestSeqRef.current = requestSeq;

    try {
      const detail = await commands.vaultGetCipherDetail({
        cipherId: normalizedCipherId,
      });

      if (requestSeq !== detailRequestSeqRef.current) {
        return;
      }

      if (detail.status === "error") {
        setCipherDetailError(errorToText(detail.error));
        return;
      }

      setSelectedCipherDetail(detail.data.cipher);
    } catch (error) {
      if (requestSeq !== detailRequestSeqRef.current) {
        return;
      }
      setCipherDetailError(errorToText(error));
    } finally {
      if (requestSeq === detailRequestSeqRef.current) {
        setIsCipherDetailLoading(false);
      }
    }
  }, []);

  const folderCipherCount = useMemo(() => {
    const map = new Map<string, number>();
    for (const cipher of viewData?.ciphers ?? []) {
      if (!cipher.folderId) {
        continue;
      }
      map.set(cipher.folderId, (map.get(cipher.folderId) ?? 0) + 1);
    }
    return map;
  }, [viewData?.ciphers]);

  const selectedMenuName = useMemo(() => {
    if (selectedMenuId === ALL_ITEMS_ID) {
      return "All items";
    }
    if (selectedMenuId === FAVORITES_ID) {
      return "Favorites";
    }
    if (selectedMenuId === TRASH_ID) {
      return "Trash";
    }
    return (
      sortedFolders.find((folder) => folder.id === selectedMenuId)?.name ??
      "Unknown folder"
    );
  }, [selectedMenuId, sortedFolders]);

  const onFolderTreeOpenChange = useCallback(
    (nodeKey: string, open: boolean) => {
      setExpandedNodeKeys((previous) => {
        const next = new Set(previous);
        if (open) {
          next.add(nodeKey);
        } else {
          next.delete(nodeKey);
        }
        return next;
      });
    },
    [],
  );

  const avatarText = useMemo(() => toAvatarText(userEmail), [userEmail]);
  const isHeaderActionBusy = isLocking || isLoggingOut || isRefreshing;
  const lockLabel = isLocking ? "锁定中..." : "锁定";
  const logoutLabel = isLoggingOut ? "登出中..." : "登出";

  return (
    <main className="flex h-dvh flex-col bg-[radial-gradient(circle_at_12%_8%,_hsl(212_95%_96%),_transparent_36%),radial-gradient(circle_at_92%_92%,_hsl(215_95%_97%),_transparent_40%),linear-gradient(145deg,_hsl(216_55%_98%),_hsl(0_0%_100%))]">
      <header
        data-tauri-drag-region
        className="w-full border-b border-slate-200/80 bg-slate-100/95 px-4 py-1 shadow-sm backdrop-blur-sm md:px-8 md:py-1.5"
      >
        <div data-tauri-drag-region className="mx-auto w-full max-w-7xl">
          <div
            data-tauri-drag-region
            className="grid gap-1.5 md:grid-cols-[minmax(0,1fr)_max-content] md:items-center"
          >
            <div
              data-tauri-drag-region
              className="relative w-full md:max-w-[420px] md:justify-self-center"
            >
              <Search className="pointer-events-none absolute top-1/2 left-3 size-4 -translate-y-1/2 text-slate-500" />
              <Input
                ref={searchInputRef}
                type="search"
                placeholder="搜索名称、ID..."
                value={headerSearchQuery}
                onChange={(event) => setHeaderSearchQuery(event.target.value)}
                className="h-8 pl-9 text-sm"
              />
            </div>

            <div data-tauri-drag-region className="md:justify-self-end">
              <DropdownMenu>
                <DropdownMenuTrigger asChild>
                  <Button
                    type="button"
                    variant="ghost"
                    className="h-auto w-full justify-start gap-2 rounded-lg border border-transparent px-2 py-1 transition-colors hover:border-slate-300 hover:bg-slate-200/70 data-[state=open]:border-slate-300 data-[state=open]:bg-slate-200/70 md:w-fit"
                  >
                    <div className="flex size-9 items-center justify-center rounded-full bg-sky-100 text-xs font-semibold text-sky-700">
                      {avatarText === "??" ? (
                        <UserRound className="size-4" />
                      ) : (
                        avatarText
                      )}
                    </div>
                    <div className="min-w-0 text-left">
                      <div className="truncate text-sm font-medium text-slate-900">
                        {userEmail}
                      </div>
                      <div className="truncate text-xs text-slate-600">
                        {userBaseUrl}
                      </div>
                    </div>
                    <ChevronDown className="ml-1 size-4 text-slate-500" />
                  </Button>
                </DropdownMenuTrigger>
                <DropdownMenuContent align="end" className="w-72">
                  <DropdownMenuLabel className="space-y-1">
                    <div className="truncate text-sm text-slate-900">
                      {userEmail}
                    </div>
                    <div className="truncate text-xs text-slate-600">
                      {userBaseUrl}
                    </div>
                  </DropdownMenuLabel>
                  <DropdownMenuSeparator />
                  <DropdownMenuItem
                    disabled={isHeaderActionBusy}
                    onSelect={() => {
                      void onLock();
                    }}
                  >
                    {isLocking ? (
                      <LoaderCircle className="animate-spin" />
                    ) : (
                      <Lock />
                    )}
                    {lockLabel}
                  </DropdownMenuItem>
                  <DropdownMenuItem
                    variant="destructive"
                    disabled={isHeaderActionBusy}
                    onSelect={() => {
                      void onLogout();
                    }}
                  >
                    {isLoggingOut ? (
                      <LoaderCircle className="animate-spin" />
                    ) : (
                      <LogOut />
                    )}
                    {logoutLabel}
                  </DropdownMenuItem>
                </DropdownMenuContent>
              </DropdownMenu>
            </div>
          </div>
        </div>
      </header>

      <section className="mx-auto flex w-full max-w-7xl min-h-0 flex-1 flex-col gap-4">
        {pageState === "loading" && (
          <div className="rounded-xl bg-white/85 px-4 py-3 text-sm text-slate-700 shadow-sm">
            正在加载 vault 数据...
          </div>
        )}

        {pageState === "needsLogin" && (
          <div className="rounded-xl bg-white/85 px-4 py-3 text-sm text-slate-700 shadow-sm">
            当前未登录，请先登录后访问 vault。
          </div>
        )}

        {pageState === "locked" && (
          <div className="rounded-xl bg-white/85 px-4 py-3 text-sm text-slate-700 shadow-sm">
            当前 vault 已锁定，请先解锁。
          </div>
        )}

        {pageState === "error" && (
          <div className="rounded-xl bg-white/85 px-4 py-3 text-sm text-red-700 shadow-sm">
            {errorText || "读取 vault 数据失败。"}
          </div>
        )}

        {pageState === "ready" && viewData && (
          <div className="grid min-h-0 flex-1 grid-cols-[220px_minmax(0,1fr)_minmax(0,1fr)] grid-rows-1 gap-0">
            <aside className="h-full min-h-0 border border-slate-300/35 bg-slate-300/30 shadow-sm backdrop-blur-md">
              <ScrollArea className="h-full">
                <div className="space-y-2 p-2">
                  <button
                    type="button"
                    onClick={() => setSelectedMenuId(ALL_ITEMS_ID)}
                    className={[
                      "w-full rounded-lg px-3 py-2 text-left text-sm transition-colors",
                      selectedMenuId === ALL_ITEMS_ID
                        ? "bg-sky-100/90 font-medium text-sky-800"
                        : "text-slate-700 hover:bg-slate-100",
                    ].join(" ")}
                  >
                    <div className="flex items-center justify-between gap-2">
                      <span className="inline-flex items-center gap-2">
                        <Archive className="size-4 text-slate-500" />
                        All items
                      </span>
                      <span className="text-xs text-slate-500">
                        {viewData.ciphers.length}
                      </span>
                    </div>
                  </button>

                  <button
                    type="button"
                    onClick={() => setSelectedMenuId(FAVORITES_ID)}
                    className={[
                      "w-full rounded-lg px-3 py-2 text-left text-sm transition-colors",
                      selectedMenuId === FAVORITES_ID
                        ? "bg-sky-100/90 font-medium text-sky-800"
                        : "text-slate-700 hover:bg-slate-100",
                    ].join(" ")}
                  >
                    <div className="flex items-center justify-between gap-2">
                      <span className="inline-flex items-center gap-2">
                        <Star className="size-4 text-slate-500" />
                        Favorites
                      </span>
                      <span className="text-xs text-slate-500">-</span>
                    </div>
                  </button>

                  <button
                    type="button"
                    onClick={() => setSelectedMenuId(TRASH_ID)}
                    className={[
                      "w-full rounded-lg px-3 py-2 text-left text-sm transition-colors",
                      selectedMenuId === TRASH_ID
                        ? "bg-sky-100/90 font-medium text-sky-800"
                        : "text-slate-700 hover:bg-slate-100",
                    ].join(" ")}
                  >
                    <div className="flex items-center justify-between gap-2">
                      <span className="inline-flex items-center gap-2">
                        <Trash2 className="size-4 text-slate-500" />
                        Trash
                      </span>
                      <span className="text-xs text-slate-500">-</span>
                    </div>
                  </button>

                  <div className="space-y-1">
                    {folderTree.map((node) => (
                      <FolderTreeMenuItem
                        key={node.key}
                        node={node}
                        depth={0}
                        selectedMenuId={selectedMenuId}
                        expandedNodeKeys={expandedNodeKeys}
                        folderCipherCount={folderCipherCount}
                        onFolderSelect={setSelectedMenuId}
                        onOpenChange={onFolderTreeOpenChange}
                      />
                    ))}
                  </div>
                </div>
              </ScrollArea>
            </aside>

            <section className="flex h-full min-h-0 flex-col bg-white/80 shadow-sm">
              <div className="flex items-center justify-between gap-2 px-2 pt-2 pb-1">
                <div className="relative h-8 min-w-0 flex-1">
                  <div className="absolute inset-y-0 left-0 flex items-center">
                    <DropdownMenu>
                      <DropdownMenuTrigger asChild>
                        <Button
                          type="button"
                          variant="ghost"
                          className="h-8 justify-start gap-1 rounded-md px-2 text-xs font-medium text-slate-700 hover:bg-slate-200/70 data-[state=open]:bg-slate-200/70"
                          aria-label="筛选类型"
                          title="筛选类型"
                        >
                          <span>{toTypeFilterLabel(typeFilter)}</span>
                          <ChevronDown className="size-3.5 text-slate-500" />
                        </Button>
                      </DropdownMenuTrigger>
                      <DropdownMenuContent align="start" className="w-44">
                        <DropdownMenuRadioGroup
                          value={typeFilter}
                          onValueChange={(value) =>
                            setTypeFilter(value as CipherTypeFilter)
                          }
                        >
                          <DropdownMenuRadioItem value="all">
                            All types
                          </DropdownMenuRadioItem>
                          <DropdownMenuRadioItem value="login">
                            Login
                          </DropdownMenuRadioItem>
                          <DropdownMenuRadioItem value="card">
                            Card
                          </DropdownMenuRadioItem>
                          <DropdownMenuRadioItem value="identify">
                            Identify
                          </DropdownMenuRadioItem>
                          <DropdownMenuRadioItem value="note">
                            Note
                          </DropdownMenuRadioItem>
                          <DropdownMenuRadioItem value="ssh_key">
                            SSH key
                          </DropdownMenuRadioItem>
                        </DropdownMenuRadioGroup>
                      </DropdownMenuContent>
                    </DropdownMenu>
                  </div>

                  <AnimatePresence>
                    {isInlineSearchOpen && (
                      <motion.div
                        className="absolute inset-y-0 right-0 left-2 z-20"
                        initial={{ opacity: 0, x: -10 }}
                        animate={{ opacity: 1, x: 0 }}
                        exit={{ opacity: 0, x: -10 }}
                        transition={{ duration: 0.18, ease: "easeOut" }}
                      >
                        <Input
                          ref={inlineSearchInputRef}
                          type="search"
                          value={cipherSearchQuery}
                          onChange={(event) =>
                            setCipherSearchQuery(event.target.value)
                          }
                          onKeyDown={(event) => {
                            if (event.key === "Escape") {
                              setCipherSearchQuery("");
                              setIsInlineSearchOpen(false);
                            }
                          }}
                          placeholder={`在 ${selectedMenuName} 中查找`}
                          className="h-8 bg-white text-xs"
                        />
                      </motion.div>
                    )}
                  </AnimatePresence>
                </div>

                <div className="flex items-center gap-1">
                  <Button
                    type="button"
                    variant="ghost"
                    className="size-8 px-0"
                    aria-label={
                      isInlineSearchOpen
                        ? "关闭搜索"
                        : `在 ${selectedMenuName} 中查找`
                    }
                    title={
                      isInlineSearchOpen
                        ? "关闭搜索"
                        : `在 ${selectedMenuName} 中查找`
                    }
                    onClick={() => {
                      if (isInlineSearchOpen) {
                        setCipherSearchQuery("");
                        setIsInlineSearchOpen(false);
                        return;
                      }
                      setIsInlineSearchOpen(true);
                    }}
                  >
                    <AnimatePresence mode="wait" initial={false}>
                      {isInlineSearchOpen ? (
                        <motion.span
                          key="close"
                          initial={{ opacity: 0, rotate: -90, scale: 0.85 }}
                          animate={{ opacity: 1, rotate: 0, scale: 1 }}
                          exit={{ opacity: 0, rotate: 90, scale: 0.85 }}
                          transition={{ duration: 0.16, ease: "easeOut" }}
                          className="inline-flex"
                        >
                          <X className="size-4" />
                        </motion.span>
                      ) : (
                        <motion.span
                          key="search"
                          initial={{ opacity: 0, rotate: 90, scale: 0.85 }}
                          animate={{ opacity: 1, rotate: 0, scale: 1 }}
                          exit={{ opacity: 0, rotate: -90, scale: 0.85 }}
                          transition={{ duration: 0.16, ease: "easeOut" }}
                          className="inline-flex"
                        >
                          <Search className="size-4" />
                        </motion.span>
                      )}
                    </AnimatePresence>
                  </Button>
                  <DropdownMenu>
                    <DropdownMenuTrigger asChild>
                      <Button
                        type="button"
                        variant="ghost"
                        className="size-8 px-0"
                        aria-label="排序"
                        title="排序"
                      >
                        <ArrowUpDown className="size-4" />
                      </Button>
                    </DropdownMenuTrigger>
                    <DropdownMenuContent align="end" className="w-44">
                      <DropdownMenuLabel>排序方式</DropdownMenuLabel>
                      <DropdownMenuSeparator />
                      <DropdownMenuRadioGroup
                        value={sortBy}
                        onValueChange={(value) =>
                          setSortBy(value as CipherSortBy)
                        }
                      >
                        <DropdownMenuRadioItem value="title">
                          标题
                        </DropdownMenuRadioItem>
                        <DropdownMenuRadioItem value="created">
                          创建日期
                        </DropdownMenuRadioItem>
                        <DropdownMenuRadioItem value="modified">
                          修改日期
                        </DropdownMenuRadioItem>
                      </DropdownMenuRadioGroup>
                      <DropdownMenuSeparator />
                      <DropdownMenuLabel>排序方向</DropdownMenuLabel>
                      <DropdownMenuRadioGroup
                        value={sortDirection}
                        onValueChange={(value) =>
                          setSortDirection(value as CipherSortDirection)
                        }
                      >
                        {sortBy === "title" ? (
                          <>
                            <DropdownMenuRadioItem value="asc">
                              按字母顺序
                            </DropdownMenuRadioItem>
                            <DropdownMenuRadioItem value="desc">
                              字母倒序
                            </DropdownMenuRadioItem>
                          </>
                        ) : (
                          <>
                            <DropdownMenuRadioItem value="desc">
                              最新的在前
                            </DropdownMenuRadioItem>
                            <DropdownMenuRadioItem value="asc">
                              最早的在前
                            </DropdownMenuRadioItem>
                          </>
                        )}
                      </DropdownMenuRadioGroup>
                    </DropdownMenuContent>
                  </DropdownMenu>
                </div>
              </div>

              <div className="px-4 pt-3 pb-2 text-sm font-medium text-slate-700">
                {selectedMenuName} ({filteredCiphers.length})
              </div>
              <ScrollArea className="min-h-0 flex-1">
                <div className="space-y-1 px-2 pb-2">
                  {filteredCiphers.map((cipher) => (
                    <CipherRow
                      key={cipher.id}
                      cipher={cipher}
                      selected={cipher.id === selectedCipherId}
                      loading={
                        isCipherDetailLoading && cipher.id === selectedCipherId
                      }
                      onClick={() => {
                        void loadCipherDetail(cipher.id);
                      }}
                    />
                  ))}
                  {filteredCiphers.length === 0 && (
                    <div className="rounded-lg bg-slate-100 px-3 py-2 text-sm text-slate-600">
                      当前筛选下没有 cipher。
                    </div>
                  )}
                </div>
              </ScrollArea>
            </section>

            <section className="h-full min-h-0 bg-white/80 shadow-sm">
              <ScrollArea className="h-full">
                <div className="p-3">
                  {!selectedCipherId && <div className="min-h-[320px]" />}

                  {selectedCipherId && isCipherDetailLoading && (
                    <div className="flex items-center gap-2 text-sm text-slate-700">
                      <LoaderCircle className="size-4 animate-spin" />
                      正在加载 cipher 详情...
                    </div>
                  )}

                  {selectedCipherId &&
                    !isCipherDetailLoading &&
                    cipherDetailError && (
                      <div className="rounded-lg bg-red-50 px-3 py-2 text-sm text-red-700">
                        {cipherDetailError}
                      </div>
                    )}

                  {selectedCipherId &&
                    !isCipherDetailLoading &&
                    !cipherDetailError &&
                    selectedCipherDetail && (
                      <CipherDetailPanel cipher={selectedCipherDetail} />
                    )}
                </div>
              </ScrollArea>
            </section>
          </div>
        )}
      </section>
    </main>
  );
}

function FolderTreeMenuItem({
  node,
  depth,
  selectedMenuId,
  expandedNodeKeys,
  folderCipherCount,
  onFolderSelect,
  onOpenChange,
}: {
  node: FolderTreeNode;
  depth: number;
  selectedMenuId: string;
  expandedNodeKeys: Set<string>;
  folderCipherCount: Map<string, number>;
  onFolderSelect: (folderId: string) => void;
  onOpenChange: (nodeKey: string, open: boolean) => void;
}) {
  const hasChildren = node.children.length > 0;
  const isExpanded = expandedNodeKeys.has(node.key);
  const isSelected = node.folderId != null && selectedMenuId === node.folderId;
  const count = countNodeCiphers(node, folderCipherCount);

  return (
    <Collapsible
      open={hasChildren ? isExpanded : false}
      onOpenChange={(open) => onOpenChange(node.key, open)}
    >
      <div className="space-y-1" style={{ paddingLeft: `${depth * 10}px` }}>
        <div className="flex items-center gap-1">
          {hasChildren ? (
            <CollapsibleTrigger asChild>
              <button
                type="button"
                className="flex size-6 items-center justify-center rounded text-slate-500 hover:bg-slate-100"
                aria-label={isExpanded ? "collapse folder" : "expand folder"}
              >
                <ChevronRight
                  className={[
                    "size-4 transition-transform",
                    isExpanded ? "rotate-90" : "",
                  ].join(" ")}
                />
              </button>
            </CollapsibleTrigger>
          ) : (
            <span className="inline-block size-6" aria-hidden="true" />
          )}

          <button
            type="button"
            onClick={() => {
              if (node.folderId) {
                onFolderSelect(node.folderId);
                return;
              }
              if (hasChildren) {
                onOpenChange(node.key, !isExpanded);
              }
            }}
            className={[
              "flex min-w-0 flex-1 items-center justify-between gap-2 rounded-lg px-2 py-1.5 text-left text-sm transition-colors",
              isSelected
                ? "bg-sky-100/90 font-medium text-sky-800"
                : "text-slate-700 hover:bg-slate-100",
            ].join(" ")}
          >
            <span className="truncate">{node.label}</span>
            <span className="text-xs text-slate-500">{count}</span>
          </button>
        </div>

        {hasChildren && (
          <CollapsibleContent>
            <div className="space-y-1">
              {node.children.map((child) => (
                <FolderTreeMenuItem
                  key={child.key}
                  node={child}
                  depth={depth + 1}
                  selectedMenuId={selectedMenuId}
                  expandedNodeKeys={expandedNodeKeys}
                  folderCipherCount={folderCipherCount}
                  onFolderSelect={onFolderSelect}
                  onOpenChange={onOpenChange}
                />
              ))}
            </div>
          </CollapsibleContent>
        )}
      </div>
    </Collapsible>
  );
}

function CipherRow({
  cipher,
  selected,
  loading,
  onClick,
}: {
  cipher: VaultCipherItemDto;
  selected: boolean;
  loading: boolean;
  onClick: () => void;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      className={[
        "w-full rounded-lg px-3 py-2 text-left transition-colors",
        selected
          ? "bg-sky-100/85 text-sky-900"
          : "bg-slate-50/80 text-slate-800 hover:bg-slate-100",
      ].join(" ")}
    >
      <div className="truncate text-sm font-medium">
        {cipher.name ?? "Untitled cipher"}
      </div>
      <div className="mt-1 truncate text-xs text-slate-600">
        {cipher.username ?? ""}
      </div>
      {loading && (
        <div className="mt-1 flex items-center gap-1 text-xs text-sky-700">
          <LoaderCircle className="size-3 animate-spin" />
          加载中...
        </div>
      )}
    </button>
  );
}

function DetailRow({
  label,
  value,
}: {
  label: string;
  value: string | null | undefined;
}) {
  if (!value) {
    return null;
  }
  return (
    <div className="grid grid-cols-[88px_minmax(0,1fr)] gap-2 text-sm">
      <div className="text-slate-500">{label}</div>
      <div className="break-all text-slate-800">{value}</div>
    </div>
  );
}

function CipherDetailPanel({ cipher }: { cipher: VaultCipherDetailDto }) {
  const username = firstNonEmptyText(
    cipher.login?.username,
    cipher.data?.username,
  );
  const uri = firstNonEmptyText(
    cipher.login?.uri,
    cipher.data?.uri,
    cipher.login?.uris[0]?.uri,
    cipher.data?.uris[0]?.uri,
  );
  const notes = firstNonEmptyText(cipher.notes, cipher.data?.notes);
  const folderId = cipher.folderId;
  const organizationId = cipher.organizationId;

  return (
    <div className="space-y-3">
      <div className="space-y-1">
        <div className="text-base font-semibold text-slate-900">
          {cipher.name ?? "Untitled cipher"}
        </div>
        <div className="text-xs text-slate-500">{cipher.id}</div>
      </div>

      <div className="space-y-2">
        <DetailRow label="Type" value={toCipherTypeLabel(cipher.type)} />
        <DetailRow label="Folder" value={folderId} />
        <DetailRow label="Org" value={organizationId} />
        <DetailRow
          label="Revision"
          value={formatRevisionDate(cipher.revisionDate)}
        />
        <DetailRow label="Username" value={username} />
        <DetailRow label="URI" value={uri} />
        <DetailRow
          label="Attachments"
          value={String(cipher.attachments.length)}
        />
      </div>

      {notes && (
        <div className="rounded-lg bg-slate-50/90 p-2">
          <div className="mb-1 text-xs text-slate-500">Notes</div>
          <pre className="whitespace-pre-wrap text-sm text-slate-800">
            {notes}
          </pre>
        </div>
      )}
    </div>
  );
}
