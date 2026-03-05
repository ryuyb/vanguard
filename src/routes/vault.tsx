import { createFileRoute, redirect, useNavigate } from "@tanstack/react-router";
import {
  Archive,
  ArrowUpDown,
  ChevronDown,
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
  type VaultViewDataResponseDto,
} from "@/bindings";
import { Button } from "@/components/ui/button";
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
import {
  ResizableHandle,
  ResizablePanel,
  ResizablePanelGroup,
} from "@/components/ui/resizable";
import { ScrollArea } from "@/components/ui/scroll-area";
import { CipherDetailPanel } from "@/features/vault/components/cipher-detail-panel";
import { CipherRow } from "@/features/vault/components/cipher-row";
import { FolderTreeMenuItem } from "@/features/vault/components/folder-tree-menu-item";
import {
  ALL_ITEMS_ID,
  FAVORITES_ID,
  TRASH_ID,
} from "@/features/vault/constants";
import type {
  CipherSortBy,
  CipherSortDirection,
  CipherTypeFilter,
  VaultPageState,
} from "@/features/vault/types";
import {
  buildFolderTree,
  collectFolderTreeKeys,
  sortFolders,
  toAvatarText,
  toSortableDate,
  toTypeFilterLabel,
  toVaultErrorText,
} from "@/features/vault/utils";
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
      const target = await resolveSessionRoute();
      if (target !== "/vault") {
        await navigate({ to: target });
        return;
      }

      const restore = await commands.authRestoreState({});
      if (restore.status === "error") {
        setPageState("error");
        setErrorText(toVaultErrorText(restore.error));
        setUserEmail("未登录");
        setUserBaseUrl("未知服务");
        return;
      }

      setUserEmail(restore.data.email ?? "未登录");
      setUserBaseUrl(restore.data.baseUrl ?? "未知服务");

      const result = await commands.vaultGetViewData();

      if (result.status === "error") {
        const text = toVaultErrorText(result.error);
        if (text.toLowerCase().includes("vault is locked")) {
          await navigate({ to: "/unlock" });
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
      setErrorText(toVaultErrorText(error));
      setViewData(null);
    } finally {
      setIsRefreshing(false);
    }
  }, [navigate]);

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
        setCipherDetailError(toVaultErrorText(detail.error));
        return;
      }

      setSelectedCipherDetail(detail.data.cipher);
    } catch (error) {
      if (requestSeq !== detailRequestSeqRef.current) {
        return;
      }
      setCipherDetailError(toVaultErrorText(error));
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
    <main className="flex h-dvh flex-col bg-[radial-gradient(circle_at_12%_8%,hsl(212_95%_96%),transparent_36%),radial-gradient(circle_at_92%_92%,hsl(215_95%_97%),transparent_40%),linear-gradient(145deg,hsl(216_55%_98%),hsl(0_0%_100%))]">
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
              className="relative w-full md:max-w-105 md:justify-self-center"
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

        {pageState === "error" && (
          <div className="rounded-xl bg-white/85 px-4 py-3 text-sm text-red-700 shadow-sm">
            {errorText || "读取 vault 数据失败。"}
          </div>
        )}

        {pageState === "ready" && viewData && (
          <ResizablePanelGroup
            orientation="horizontal"
            className="min-h-0 flex-1"
          >
            <ResizablePanel defaultSize={22} minSize={16}>
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
            </ResizablePanel>

            <ResizableHandle withHandle className="bg-slate-300/60" />

            <ResizablePanel defaultSize={39} minSize={22}>
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

                <ScrollArea className="min-h-0 flex-1">
                  <div className="space-y-1 px-2 pb-2">
                    {filteredCiphers.map((cipher) => (
                      <CipherRow
                        key={cipher.id}
                        cipher={cipher}
                        selected={cipher.id === selectedCipherId}
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
            </ResizablePanel>

            <ResizableHandle withHandle className="bg-slate-300/60" />

            <ResizablePanel defaultSize={39} minSize={24}>
              <section className="h-full min-h-0 bg-white/80 shadow-sm">
                <ScrollArea className="h-full">
                  <div className="p-3">
                    {!selectedCipherId && <div className="min-h-80" />}

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
                        <CipherDetailPanel
                          key={selectedCipherDetail.id}
                          cipher={selectedCipherDetail}
                        />
                      )}
                  </div>
                </ScrollArea>
              </section>
            </ResizablePanel>
          </ResizablePanelGroup>
        )}
      </section>
    </main>
  );
}
