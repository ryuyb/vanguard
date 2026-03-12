import {
  Archive,
  ArrowUpDown,
  ChevronDown,
  Folder,
  FolderPlus,
  LoaderCircle,
  Lock,
  LogOut,
  Search,
  Settings,
  Star,
  Trash2,
  UserRound,
  X,
} from "lucide-react";
import { AnimatePresence, motion } from "motion/react";
import { useEffect, useRef, useState } from "react";
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
import {
  ALL_ITEMS_ID,
  CipherDetailPanel,
  CipherRow,
  type CipherSortBy,
  type CipherSortDirection,
  type CipherTypeFilter,
  FAVORITES_ID,
  FolderTreeMenuItem,
  ICON_OBSERVER_CONFIG,
  NO_FOLDER_ID,
  TRASH_ID,
  toTypeFilterLabel,
  VaultSettingsDialog,
} from "@/features/vault";
import { DeleteFolderDialog } from "@/features/vault/components/delete-folder-dialog";
import { FolderDialog } from "@/features/vault/components/folder-dialog";
import { useVaultPageModel } from "@/features/vault/hooks";
import { useFolderActions } from "@/features/vault/hooks/use-folder-actions";
import type { VaultPageNavigationTarget } from "@/features/vault/hooks/use-vault-page-model";
import { getCipherIconUrl } from "@/features/vault/utils";
import { toast } from "@/lib/toast";

type VaultPageProps = {
  navigateTo: (to: VaultPageNavigationTarget) => Promise<void>;
};

function CipherRowObserver({
  children,
  cipherId,
  onVisibilityChange,
}: {
  children: React.ReactNode;
  cipherId: string;
  onVisibilityChange: (cipherId: string, visible: boolean) => void;
}) {
  const ref = useRef<HTMLDivElement | null>(null);
  const callbackRef = useRef(onVisibilityChange);

  useEffect(() => {
    callbackRef.current = onVisibilityChange;
  }, [onVisibilityChange]);

  useEffect(() => {
    const node = ref.current;
    if (!node) {
      return;
    }

    let rafId = 0;
    const observer = new IntersectionObserver(([entry]) => {
      window.cancelAnimationFrame(rafId);
      rafId = window.requestAnimationFrame(() => {
        callbackRef.current(cipherId, entry?.isIntersecting === true);
      });
    }, ICON_OBSERVER_CONFIG);

    observer.observe(node);

    return () => {
      window.cancelAnimationFrame(rafId);
      observer.disconnect();
    };
  }, [cipherId]);

  return <div ref={ref}>{children}</div>;
}

export function VaultPage({ navigateTo }: VaultPageProps) {
  const [isSettingsDialogOpen, setIsSettingsDialogOpen] = useState(false);

  // 文件夹操作状态
  const [folderDialogMode, setFolderDialogMode] = useState<"create" | "rename">(
    "create",
  );
  const [isFolderDialogOpen, setIsFolderDialogOpen] = useState(false);
  const [isDeleteDialogOpen, setIsDeleteDialogOpen] = useState(false);
  const [selectedFolderId, setSelectedFolderId] = useState<string | null>(null);
  const [selectedFolderName, setSelectedFolderName] = useState("");
  const [parentFolderName, setParentFolderName] = useState<string | null>(null);

  const {
    avatarText,
    cipherDetailError,
    cipherSearchQuery,
    errorText,
    expandedNodeKeys,
    favoriteCipherCount,
    filteredCiphers,
    folderCipherCount,
    folderTree,
    headerSearchQuery,
    iconServer,
    inlineSearchInputRef,
    isCipherDetailLoading,
    isHeaderActionBusy,
    isInlineSearchOpen,
    isLocking,
    isLoggingOut,
    loadCipherDetail,
    loadVaultData,
    lockLabel,
    logoutLabel,
    markCipherIconFallback,
    markCipherIconLoaded,
    noFolderCipherCount,
    onFolderTreeOpenChange,
    onLock,
    onLogout,
    pageState,
    searchInputRef,
    selectedCipherDetail,
    selectedCipherId,
    selectedMenuId,
    selectedMenuName,
    setCipherRowVisible,
    setCipherSearchQuery,
    setHeaderSearchQuery,
    setIsInlineSearchOpen,
    setSelectedMenuId,
    setSortBy,
    setSortDirection,
    setTypeFilter,
    sortBy,
    sortDirection,
    trashCipherCount,
    typeFilter,
    userBaseUrl,
    userEmail,
    viewData,
  } = useVaultPageModel({ navigateTo });

  const folderActions = useFolderActions({
    onSuccess: () => {
      void loadVaultData();
    },
  });

  // 文件夹操作处理函数
  const handleCreateFolder = () => {
    setFolderDialogMode("create");
    setSelectedFolderName("");
    setParentFolderName(null);
    setIsFolderDialogOpen(true);
  };

  const handleCreateSubFolder = (parentPath: string) => {
    setFolderDialogMode("create");
    setSelectedFolderName("");
    setParentFolderName(parentPath);
    setIsFolderDialogOpen(true);
  };

  const handleRenameFolder = (folderId: string, currentName: string) => {
    setFolderDialogMode("rename");
    setSelectedFolderId(folderId);
    setSelectedFolderName(currentName);
    setParentFolderName(null);
    setIsFolderDialogOpen(true);
  };

  const handleDeleteFolder = (folderId: string, folderName: string) => {
    setSelectedFolderId(folderId);
    setSelectedFolderName(folderName);
    setIsDeleteDialogOpen(true);
  };

  const handleFolderDialogConfirm = (name: string) => {
    if (folderDialogMode === "create") {
      // 如果有父文件夹,创建子文件夹(通过名称前缀)
      const fullName = parentFolderName ? `${parentFolderName}/${name}` : name;
      folderActions.createFolder.mutate(fullName, {
        onSuccess: () => {
          setIsFolderDialogOpen(false);
          toast.success("创建成功", {
            description: `文件夹 "${name}" 已创建`,
          });
        },
        onError: (error) => {
          toast.error("创建失败", {
            description:
              error instanceof Error ? error.message : "创建文件夹时发生错误",
          });
        },
      });
    } else if (folderDialogMode === "rename" && selectedFolderId) {
      folderActions.renameFolder.mutate(
        { folderId: selectedFolderId, newName: name },
        {
          onSuccess: () => {
            setIsFolderDialogOpen(false);
            toast.success("重命名成功", {
              description: `文件夹已重命名为 "${name}"`,
            });
          },
          onError: (error) => {
            toast.error("重命名失败", {
              description:
                error instanceof Error
                  ? error.message
                  : "重命名文件夹时发生错误",
            });
          },
        },
      );
    }
  };

  const handleDeleteDialogConfirm = () => {
    if (selectedFolderId) {
      folderActions.deleteFolder.mutate(selectedFolderId, {
        onSuccess: () => {
          setIsDeleteDialogOpen(false);
          toast.success("删除成功", {
            description: `文件夹 "${selectedFolderName}" 已删除`,
          });
          // 如果删除的是当前选中的文件夹,切换到 All Items
          if (selectedMenuId === selectedFolderId) {
            setSelectedMenuId(ALL_ITEMS_ID);
          }
        },
        onError: (error) => {
          toast.error("删除失败", {
            description:
              error instanceof Error ? error.message : "删除文件夹时发生错误",
          });
        },
      });
    }
  };

  return (
    <main className="flex h-dvh flex-col bg-gradient-to-br from-slate-50 via-white to-slate-100">
      <header
        data-tauri-drag-region
        className="w-full border-b border-slate-200/60 bg-white/80 px-4 py-2 shadow-sm backdrop-blur-md md:px-8 md:py-2.5"
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
              <Search className="pointer-events-none absolute top-1/2 left-3 size-4 -translate-y-1/2 text-slate-400" />
              <Input
                ref={searchInputRef}
                type="search"
                placeholder="搜索名称、ID..."
                value={headerSearchQuery}
                onChange={(event) => setHeaderSearchQuery(event.target.value)}
                className="h-9 pl-9 text-sm border-slate-200 bg-slate-50/50 focus:bg-white transition-colors"
              />
            </div>

            <div data-tauri-drag-region className="md:justify-self-end">
              <DropdownMenu>
                <DropdownMenuTrigger asChild>
                  <Button
                    type="button"
                    variant="ghost"
                    className="h-auto w-full justify-start gap-2.5 rounded-lg border border-transparent px-2.5 py-1.5 transition-all hover:border-slate-200 hover:bg-slate-50 data-[state=open]:border-slate-200 data-[state=open]:bg-slate-50 md:w-fit"
                  >
                    <div className="flex size-9 items-center justify-center rounded-full bg-gradient-to-br from-blue-500 to-blue-600 text-xs font-semibold text-white shadow-sm">
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
                    onSelect={() => {
                      setIsSettingsDialogOpen(true);
                    }}
                  >
                    <Settings />
                    设置
                  </DropdownMenuItem>
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

      <section className="mx-auto flex w-full max-w-7xl min-h-0 flex-1 flex-col gap-4 p-4">
        {pageState === "loading" && (
          <div className="rounded-xl bg-white px-4 py-3 text-sm text-slate-700 shadow-sm border border-slate-200">
            正在加载 vault 数据...
          </div>
        )}

        {pageState === "error" && (
          <div className="rounded-xl bg-white px-4 py-3 text-sm text-red-700 shadow-sm border border-red-200">
            {errorText || "读取 vault 数据失败。"}
          </div>
        )}

        {pageState === "ready" && viewData && (
          <ResizablePanelGroup
            orientation="horizontal"
            className="min-h-0 flex-1 rounded-xl overflow-hidden shadow-lg"
          >
            <ResizablePanel defaultSize={20} minSize={16}>
              <aside className="h-full min-h-0 bg-white border-r border-slate-200">
                <ScrollArea className="h-full">
                  <div className="space-y-1 p-3">
                    <button
                      type="button"
                      onClick={() => setSelectedMenuId(ALL_ITEMS_ID)}
                      className={[
                        "w-full rounded-lg px-3 py-2 text-left text-sm font-medium transition-all",
                        selectedMenuId === ALL_ITEMS_ID
                          ? "bg-blue-50 text-blue-700 shadow-sm"
                          : "text-slate-700 hover:bg-slate-50",
                      ].join(" ")}
                    >
                      <div className="flex items-center justify-between gap-2">
                        <span className="inline-flex items-center gap-2.5">
                          <Archive
                            className={[
                              "size-4",
                              selectedMenuId === ALL_ITEMS_ID
                                ? "text-blue-600"
                                : "text-slate-400",
                            ].join(" ")}
                          />
                          All items
                        </span>
                        <span
                          className={[
                            "text-xs font-semibold",
                            selectedMenuId === ALL_ITEMS_ID
                              ? "text-blue-600"
                              : "text-slate-400",
                          ].join(" ")}
                        >
                          {
                            (viewData.ciphers ?? []).filter(
                              (cipher) => cipher.deletedDate == null,
                            ).length
                          }
                        </span>
                      </div>
                    </button>

                    <button
                      type="button"
                      onClick={() => setSelectedMenuId(FAVORITES_ID)}
                      className={[
                        "w-full rounded-lg px-3 py-2 text-left text-sm font-medium transition-all",
                        selectedMenuId === FAVORITES_ID
                          ? "bg-blue-50 text-blue-700 shadow-sm"
                          : "text-slate-700 hover:bg-slate-50",
                      ].join(" ")}
                    >
                      <div className="flex items-center justify-between gap-2">
                        <span className="inline-flex items-center gap-2.5">
                          <Star
                            className={[
                              "size-4",
                              selectedMenuId === FAVORITES_ID
                                ? "text-blue-600"
                                : "text-slate-400",
                            ].join(" ")}
                          />
                          Favorites
                        </span>
                        <span
                          className={[
                            "text-xs font-semibold",
                            selectedMenuId === FAVORITES_ID
                              ? "text-blue-600"
                              : "text-slate-400",
                          ].join(" ")}
                        >
                          {favoriteCipherCount}
                        </span>
                      </div>
                    </button>

                    <button
                      type="button"
                      onClick={() => setSelectedMenuId(TRASH_ID)}
                      className={[
                        "w-full rounded-lg px-3 py-2 text-left text-sm font-medium transition-all",
                        selectedMenuId === TRASH_ID
                          ? "bg-blue-50 text-blue-700 shadow-sm"
                          : "text-slate-700 hover:bg-slate-50",
                      ].join(" ")}
                    >
                      <div className="flex items-center justify-between gap-2">
                        <span className="inline-flex items-center gap-2.5">
                          <Trash2
                            className={[
                              "size-4",
                              selectedMenuId === TRASH_ID
                                ? "text-blue-600"
                                : "text-slate-400",
                            ].join(" ")}
                          />
                          Trash
                        </span>
                        <span
                          className={[
                            "text-xs font-semibold",
                            selectedMenuId === TRASH_ID
                              ? "text-blue-600"
                              : "text-slate-400",
                          ].join(" ")}
                        >
                          {trashCipherCount}
                        </span>
                      </div>
                    </button>

                    <div className="mt-4 mb-2 flex items-center justify-between px-3">
                      <h3 className="text-xs font-semibold text-slate-500 uppercase tracking-wider">
                        文件夹
                      </h3>
                      <Button
                        type="button"
                        variant="ghost"
                        size="sm"
                        onClick={handleCreateFolder}
                        className="h-6 px-2 text-xs"
                      >
                        <FolderPlus className="size-3.5" />
                        新建
                      </Button>
                    </div>

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
                          onRenameFolder={handleRenameFolder}
                          onDeleteFolder={handleDeleteFolder}
                          onCreateSubFolder={handleCreateSubFolder}
                        />
                      ))}

                      <div className="flex items-center gap-1">
                        <span className="inline-flex size-6 items-center justify-center" aria-hidden="true">
                          <Folder className="size-4 text-slate-400" />
                        </span>
                        <button
                          type="button"
                          onClick={() => setSelectedMenuId(NO_FOLDER_ID)}
                          className={[
                            "flex min-w-0 flex-1 items-center justify-between gap-2 rounded-lg px-2 py-1.5 text-left text-sm font-medium transition-all",
                            selectedMenuId === NO_FOLDER_ID
                              ? "bg-blue-50 text-blue-700 shadow-sm"
                              : "text-slate-700 hover:bg-slate-50",
                          ].join(" ")}
                        >
                          <span className="truncate">无文件夹</span>
                          <span
                            className={[
                              "text-xs font-semibold",
                              selectedMenuId === NO_FOLDER_ID
                                ? "text-blue-600"
                                : "text-slate-400",
                            ].join(" ")}
                          >
                            {noFolderCipherCount}
                          </span>
                        </button>
                      </div>
                    </div>
                  </div>
                </ScrollArea>
              </aside>
            </ResizablePanel>

            <ResizableHandle withHandle className="bg-slate-200" />

            <ResizablePanel defaultSize={40} minSize={24}>
              <section className="flex h-full min-h-0 flex-col bg-slate-50/50 border-r border-slate-200">
                <div className="flex items-center justify-between gap-2 px-3 pt-3 pb-2 bg-white border-b border-slate-200">
                  <div className="relative h-8 min-w-0 flex-1">
                    <div className="absolute inset-y-0 left-0 flex items-center">
                      <DropdownMenu>
                        <DropdownMenuTrigger asChild>
                          <Button
                            type="button"
                            variant="ghost"
                            className="h-8 justify-start gap-1 rounded-md px-2.5 text-xs font-medium text-slate-700 hover:bg-slate-100 data-[state=open]:bg-slate-100"
                            aria-label="筛选类型"
                            title="筛选类型"
                          >
                            <span>{toTypeFilterLabel(typeFilter)}</span>
                            <ChevronDown className="size-3.5 text-slate-400" />
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
                            className="h-8 bg-white text-xs border-slate-200"
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
                  <div className="space-y-1.5 px-3 py-2">
                    {filteredCiphers.map(
                      (cipher: (typeof filteredCiphers)[number]) => (
                        <CipherRowObserver
                          key={cipher.id}
                          cipherId={cipher.id}
                          onVisibilityChange={setCipherRowVisible}
                        >
                          <CipherRow
                            cipher={cipher}
                            iconLoadState={cipher.iconLoadState}
                            onClick={() => {
                              void loadCipherDetail(cipher.id);
                            }}
                            onIconError={() => {
                              markCipherIconFallback(cipher.id);
                            }}
                            onIconLoad={() => {
                              markCipherIconLoaded(cipher.id);
                            }}
                            selected={cipher.id === selectedCipherId}
                            shouldLoadIcon={cipher.shouldLoadIcon}
                          />
                        </CipherRowObserver>
                      ),
                    )}
                    {filteredCiphers.length === 0 && (
                      <div className="rounded-lg bg-white border border-slate-200 px-3 py-2 text-sm text-slate-600">
                        当前筛选下没有 cipher。
                      </div>
                    )}
                  </div>
                </ScrollArea>
              </section>
            </ResizablePanel>

            <ResizableHandle withHandle className="bg-slate-200" />

            <ResizablePanel defaultSize={40} minSize={26}>
              <section className="h-full min-h-0 min-w-0 overflow-x-hidden bg-white">
                <ScrollArea className="h-full min-w-0 [&>[data-slot=scroll-area-viewport]]:min-w-0 [&>[data-slot=scroll-area-viewport]]:overflow-x-hidden [&>[data-slot=scroll-area-viewport]>div]:!block [&>[data-slot=scroll-area-viewport]>div]:h-full [&>[data-slot=scroll-area-viewport]>div]:min-w-0 [&>[data-slot=scroll-area-viewport]>div]:w-full">
                  <div className="flex h-full min-w-0 w-full flex-col p-3">
                    {!selectedCipherId && <div className="min-h-80" />}

                    {selectedCipherId && isCipherDetailLoading && (
                      <div className="flex items-center gap-2 text-sm text-slate-700 bg-white rounded-lg border border-slate-200 px-3 py-2">
                        <LoaderCircle className="size-4 animate-spin text-blue-600" />
                        正在加载 cipher 详情...
                      </div>
                    )}

                    {selectedCipherId &&
                      !isCipherDetailLoading &&
                      cipherDetailError && (
                        <div className="rounded-lg bg-red-50 border border-red-200 px-3 py-2 text-sm text-red-700">
                          {cipherDetailError}
                        </div>
                      )}

                    {selectedCipherId &&
                      !isCipherDetailLoading &&
                      !cipherDetailError &&
                      selectedCipherDetail && (
                        <div className="min-h-0 min-w-0 w-full flex-1 overflow-x-hidden">
                          <CipherDetailPanel
                            key={selectedCipherDetail.id}
                            cipher={selectedCipherDetail}
                            iconServer={iconServer}
                            iconUrl={getCipherIconUrl(
                              selectedCipherDetail,
                              iconServer,
                            )}
                          />
                        </div>
                      )}
                  </div>
                </ScrollArea>
              </section>
            </ResizablePanel>
          </ResizablePanelGroup>
        )}
      </section>

      <VaultSettingsDialog
        open={isSettingsDialogOpen}
        onOpenChange={setIsSettingsDialogOpen}
      />

      <FolderDialog
        open={isFolderDialogOpen}
        mode={folderDialogMode}
        initialName={selectedFolderName}
        parentFolderName={parentFolderName}
        onOpenChange={setIsFolderDialogOpen}
        onConfirm={handleFolderDialogConfirm}
        isLoading={
          folderDialogMode === "create"
            ? folderActions.createFolder.isLoading
            : folderActions.renameFolder.isLoading
        }
      />

      <DeleteFolderDialog
        open={isDeleteDialogOpen}
        folderName={selectedFolderName}
        onOpenChange={setIsDeleteDialogOpen}
        onConfirm={handleDeleteDialogConfirm}
        isLoading={folderActions.deleteFolder.isLoading}
      />
    </main>
  );
}
