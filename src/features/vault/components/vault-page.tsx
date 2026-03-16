import {
  AlertTriangle,
  Archive,
  ArrowUpDown,
  ChevronDown,
  Copy,
  Edit2,
  Eye,
  Folder,
  FolderPlus,
  LoaderCircle,
  Lock,
  LogOut,
  MoreVertical,
  Plus,
  RotateCcw,
  Search,
  Settings,
  Star,
  Trash2,
  UserRound,
  X,
} from "lucide-react";
import { AnimatePresence, motion } from "motion/react";
import { useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import {
  commands,
  type SyncCipher,
  type VaultCipherDetailDto,
} from "@/bindings";
import { Button } from "@/components/ui/button";
import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuTrigger,
} from "@/components/ui/context-menu";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
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
import { CipherFormDialog } from "@/features/vault/components/cipher-form-dialog";
import { DeleteCipherDialog } from "@/features/vault/components/delete-cipher-dialog";
import { DeleteFolderDialog } from "@/features/vault/components/delete-folder-dialog";
import { FolderDialog } from "@/features/vault/components/folder-dialog";
import { useVaultPageModel } from "@/features/vault/hooks";
import { useCipherMutations } from "@/features/vault/hooks/use-cipher-mutations";
import { useFolderActions } from "@/features/vault/hooks/use-folder-actions";
import type { VaultPageNavigationTarget } from "@/features/vault/hooks/use-vault-page-model";
import { getCipherIconUrl } from "@/features/vault/utils";
import { vaultCipherDetailToSyncCipher } from "@/features/vault/utils/cipher-converter";
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
  const { t } = useTranslation();
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

  // Cipher 操作状态
  const [cipherFormMode, setCipherFormMode] = useState<"create" | "edit">(
    "create",
  );
  const [isCipherFormOpen, setIsCipherFormOpen] = useState(false);
  const [isDeleteCipherDialogOpen, setIsDeleteCipherDialogOpen] =
    useState(false);
  const [selectedCipherForEdit, setSelectedCipherForEdit] =
    useState<SyncCipher | null>(null);
  const [selectedCipherIdForDelete, setSelectedCipherIdForDelete] = useState<
    string | null
  >(null);
  const [selectedCipherNameForDelete, setSelectedCipherNameForDelete] =
    useState("");

  // 回收站操作状态
  const [isRestoreDialogOpen, setIsRestoreDialogOpen] = useState(false);
  const [isPermanentDeleteDialogOpen, setIsPermanentDeleteDialogOpen] =
    useState(false);
  const [trashActionCipherId, setTrashActionCipherId] = useState("");
  const [trashActionCipherName, setTrashActionCipherName] = useState("");
  const [isTrashActionLoading, setIsTrashActionLoading] = useState(false);

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

  const cipherMutations = useCipherMutations({
    onSuccess: () => {
      // 成功后会自动触发事件，由 useCipherEvents 处理刷新
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
          toast.success(t("vault.feedback.folder.createSuccess.title"), {
            description: t("vault.feedback.folder.createSuccess.description", {
              name,
            }),
          });
        },
        onError: (error) => {
          toast.error(t("vault.feedback.folder.createError.title"), {
            description:
              error instanceof Error
                ? error.message
                : t("vault.feedback.folder.createError.description"),
          });
        },
      });
    } else if (folderDialogMode === "rename" && selectedFolderId) {
      folderActions.renameFolder.mutate(
        { folderId: selectedFolderId, newName: name },
        {
          onSuccess: () => {
            setIsFolderDialogOpen(false);
            toast.success(t("vault.feedback.folder.renameSuccess.title"), {
              description: t(
                "vault.feedback.folder.renameSuccess.description",
                {
                  name,
                },
              ),
            });
          },
          onError: (error) => {
            toast.error(t("vault.feedback.folder.renameError.title"), {
              description:
                error instanceof Error
                  ? error.message
                  : t("vault.feedback.folder.renameError.description"),
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
          toast.success(t("vault.feedback.folder.deleteSuccess.title"), {
            description: t("vault.feedback.folder.deleteSuccess.description", {
              name: selectedFolderName,
            }),
          });
          // 如果删除的是当前选中的文件夹,切换到 All Items
          if (selectedMenuId === selectedFolderId) {
            setSelectedMenuId(ALL_ITEMS_ID);
          }
        },
        onError: (error) => {
          toast.error(t("vault.feedback.folder.deleteError.title"), {
            description:
              error instanceof Error
                ? error.message
                : t("vault.feedback.folder.deleteError.description"),
          });
        },
      });
    }
  };

  // Cipher 操作处理函数
  const handleCreateCipher = () => {
    setCipherFormMode("create");
    setSelectedCipherForEdit(null);
    setIsCipherFormOpen(true);
  };

  const handleEditCipher = (cipher: VaultCipherDetailDto) => {
    setCipherFormMode("edit");
    setSelectedCipherForEdit(vaultCipherDetailToSyncCipher(cipher));
    setIsCipherFormOpen(true);
  };

  const handleDeleteCipher = (cipherId: string, cipherName: string) => {
    setSelectedCipherIdForDelete(cipherId);
    setSelectedCipherNameForDelete(cipherName);
    setIsDeleteCipherDialogOpen(true);
  };

  const handleCloneCipher = async (cipherId: string) => {
    const result = await commands.vaultGetCipherDetail({ cipherId });
    if (result.status === "error") return;
    const cloned = vaultCipherDetailToSyncCipher(result.data.cipher);
    cloned.id = "";
    const suffix = t("vault.page.cipher.cloneSuffix");
    const baseName = cloned.name ?? t("vault.page.cipher.untitled");
    cloned.name = `${baseName} - ${suffix}`;
    if (cloned.data) cloned.data.name = cloned.name;
    setCipherFormMode("create");
    setSelectedCipherForEdit(cloned);
    setIsCipherFormOpen(true);
  };

  const handleCipherFormConfirm = (cipher: SyncCipher) => {
    if (cipherFormMode === "create") {
      cipherMutations.createCipher.mutate(cipher, {
        onSuccess: () => {
          setIsCipherFormOpen(false);
          toast.success(t("vault.feedback.cipher.createSuccess.title"), {
            description: t("vault.feedback.cipher.createSuccess.description", {
              name: cipher.name ?? t("vault.page.cipher.untitled"),
            }),
          });
        },
        onError: (error) => {
          toast.error(t("vault.feedback.cipher.createError.title"), {
            description:
              error instanceof Error
                ? error.message
                : t("vault.feedback.cipher.createError.description"),
          });
        },
      });
    } else if (cipherFormMode === "edit") {
      cipherMutations.updateCipher.mutate(
        { cipherId: cipher.id, cipher },
        {
          onSuccess: () => {
            setIsCipherFormOpen(false);
            toast.success(t("vault.feedback.cipher.saveSuccess.title"), {
              description: t("vault.feedback.cipher.saveSuccess.description", {
                name: cipher.name ?? t("vault.page.cipher.untitled"),
              }),
            });
          },
          onError: (error) => {
            toast.error(t("vault.feedback.cipher.saveError.title"), {
              description:
                error instanceof Error
                  ? error.message
                  : t("vault.feedback.cipher.saveError.description"),
            });
          },
        },
      );
    }
  };

  const handleDeleteCipherConfirm = () => {
    if (selectedCipherIdForDelete) {
      cipherMutations.deleteCipher.mutate(selectedCipherIdForDelete, {
        onSuccess: () => {
          setIsDeleteCipherDialogOpen(false);
          toast.success(t("vault.feedback.cipher.deleteSuccess.title"), {
            description: t("vault.feedback.cipher.deleteSuccess.description", {
              name: selectedCipherNameForDelete,
            }),
          });
        },
        onError: (error) => {
          toast.error(t("vault.feedback.cipher.deleteError.title"), {
            description:
              error instanceof Error
                ? error.message
                : t("vault.feedback.cipher.deleteError.description"),
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
                placeholder={t("vault.page.search.placeholder")}
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
                    {t("vault.page.actions.settings")}
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
            {t("vault.page.states.loading")}
          </div>
        )}

        {pageState === "error" && (
          <div className="rounded-xl bg-white px-4 py-3 text-sm text-red-700 shadow-sm border border-red-200">
            {errorText || t("vault.page.states.loadError")}
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
                          {t("vault.page.menus.allItems")}
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
                          {t("vault.page.menus.favorites")}
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
                          {t("vault.page.menus.trash")}
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
                        {t("vault.page.folders.title")}
                      </h3>
                      <Button
                        type="button"
                        variant="ghost"
                        size="sm"
                        onClick={handleCreateFolder}
                        className="h-6 px-2 text-xs"
                      >
                        <FolderPlus className="size-3.5" />
                        {t("vault.page.actions.create")}
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
                        <span
                          className="inline-flex size-6 items-center justify-center"
                          aria-hidden="true"
                        >
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
                          <span className="truncate">
                            {t("vault.page.menus.noFolder")}
                          </span>
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
                            aria-label={t("vault.page.filters.ariaLabel")}
                            title={t("vault.page.filters.ariaLabel")}
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
                              {t("vault.page.filters.types.all")}
                            </DropdownMenuRadioItem>
                            <DropdownMenuRadioItem value="login">
                              {t("vault.page.filters.types.login")}
                            </DropdownMenuRadioItem>
                            <DropdownMenuRadioItem value="card">
                              {t("vault.page.filters.types.card")}
                            </DropdownMenuRadioItem>
                            <DropdownMenuRadioItem value="identify">
                              {t("vault.page.filters.types.identity")}
                            </DropdownMenuRadioItem>
                            <DropdownMenuRadioItem value="note">
                              {t("vault.page.filters.types.note")}
                            </DropdownMenuRadioItem>
                            <DropdownMenuRadioItem value="ssh_key">
                              {t("vault.page.filters.types.sshKey")}
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
                            placeholder={t(
                              "vault.page.search.inlinePlaceholder",
                              {
                                menu: selectedMenuName,
                              },
                            )}
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
                      aria-label={t("vault.page.cipher.create")}
                      title={t("vault.page.cipher.create")}
                      onClick={handleCreateCipher}
                    >
                      <Plus className="size-4" />
                    </Button>
                    <Button
                      type="button"
                      variant="ghost"
                      className="size-8 px-0"
                      aria-label={
                        isInlineSearchOpen
                          ? t("vault.page.search.close")
                          : t("vault.page.search.inlinePlaceholder", {
                              menu: selectedMenuName,
                            })
                      }
                      title={
                        isInlineSearchOpen
                          ? t("vault.page.search.close")
                          : t("vault.page.search.inlinePlaceholder", {
                              menu: selectedMenuName,
                            })
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
                          aria-label={t("vault.page.sort.ariaLabel")}
                          title={t("vault.page.sort.ariaLabel")}
                        >
                          <ArrowUpDown className="size-4" />
                        </Button>
                      </DropdownMenuTrigger>
                      <DropdownMenuContent align="end" className="w-44">
                        <DropdownMenuLabel>
                          {t("vault.page.sort.byLabel")}
                        </DropdownMenuLabel>
                        <DropdownMenuSeparator />
                        <DropdownMenuRadioGroup
                          value={sortBy}
                          onValueChange={(value) =>
                            setSortBy(value as CipherSortBy)
                          }
                        >
                          <DropdownMenuRadioItem value="title">
                            {t("vault.page.sort.by.title")}
                          </DropdownMenuRadioItem>
                          <DropdownMenuRadioItem value="created">
                            {t("vault.page.sort.by.created")}
                          </DropdownMenuRadioItem>
                          <DropdownMenuRadioItem value="modified">
                            {t("vault.page.sort.by.modified")}
                          </DropdownMenuRadioItem>
                        </DropdownMenuRadioGroup>
                        <DropdownMenuSeparator />
                        <DropdownMenuLabel>
                          {t("vault.page.sort.directionLabel")}
                        </DropdownMenuLabel>
                        <DropdownMenuRadioGroup
                          value={sortDirection}
                          onValueChange={(value) =>
                            setSortDirection(value as CipherSortDirection)
                          }
                        >
                          {sortBy === "title" ? (
                            <>
                              <DropdownMenuRadioItem value="asc">
                                {t("vault.page.sort.direction.alphaAsc")}
                              </DropdownMenuRadioItem>
                              <DropdownMenuRadioItem value="desc">
                                {t("vault.page.sort.direction.alphaDesc")}
                              </DropdownMenuRadioItem>
                            </>
                          ) : (
                            <>
                              <DropdownMenuRadioItem value="desc">
                                {t("vault.page.sort.direction.newestFirst")}
                              </DropdownMenuRadioItem>
                              <DropdownMenuRadioItem value="asc">
                                {t("vault.page.sort.direction.oldestFirst")}
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
                          <ContextMenu>
                            <ContextMenuTrigger asChild>
                              <div>
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
                              </div>
                            </ContextMenuTrigger>
                            <ContextMenuContent className="w-44">
                              <ContextMenuItem
                                onSelect={() => {
                                  void loadCipherDetail(cipher.id);
                                }}
                              >
                                <Eye className="size-4" />
                                {t("vault.page.cipher.contextMenu.view")}
                              </ContextMenuItem>
                              {selectedMenuId !== TRASH_ID && (
                                <>
                                  <ContextMenuItem
                                    onSelect={async () => {
                                      const result =
                                        await commands.vaultGetCipherDetail({
                                          cipherId: cipher.id,
                                        });
                                      if (result.status === "error") return;
                                      handleEditCipher(result.data.cipher);
                                    }}
                                  >
                                    <Edit2 className="size-4" />
                                    {t("vault.page.cipher.contextMenu.edit")}
                                  </ContextMenuItem>
                                  <ContextMenuItem
                                    onSelect={() => {
                                      void handleCloneCipher(cipher.id);
                                    }}
                                  >
                                    <Copy className="size-4" />
                                    {t("vault.page.cipher.contextMenu.clone")}
                                  </ContextMenuItem>
                                </>
                              )}
                            </ContextMenuContent>
                          </ContextMenu>
                        </CipherRowObserver>
                      ),
                    )}
                    {filteredCiphers.length === 0 && (
                      <div className="rounded-lg bg-white border border-slate-200 px-3 py-2 text-sm text-slate-600">
                        {t("vault.page.states.emptyFiltered")}
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
                        {t("vault.page.states.loadingCipherDetail")}
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
                        <div className="min-h-0 min-w-0 w-full flex-1 overflow-x-hidden space-y-3">
                          {/* Cipher 操作栏 */}
                          {selectedMenuId !== TRASH_ID && (
                            <div className="flex items-center justify-end gap-2 px-1">
                              <DropdownMenu>
                                <DropdownMenuTrigger asChild>
                                  <Button
                                    type="button"
                                    variant="ghost"
                                    size="sm"
                                    className="h-8 px-2"
                                  >
                                    <MoreVertical className="size-4" />
                                  </Button>
                                </DropdownMenuTrigger>
                                <DropdownMenuContent
                                  align="end"
                                  className="w-44"
                                >
                                  <DropdownMenuItem
                                    onSelect={() =>
                                      handleEditCipher(selectedCipherDetail)
                                    }
                                  >
                                    <Edit2 className="size-4" />
                                    {t("vault.page.actions.edit")}
                                  </DropdownMenuItem>
                                  <DropdownMenuSeparator />
                                  <DropdownMenuItem
                                    variant="destructive"
                                    onSelect={() =>
                                      handleDeleteCipher(
                                        selectedCipherDetail.id,
                                        selectedCipherDetail.name ??
                                          t("vault.page.cipher.untitled"),
                                      )
                                    }
                                  >
                                    <Trash2 className="size-4" />
                                    {t("vault.page.actions.delete")}
                                  </DropdownMenuItem>
                                </DropdownMenuContent>
                              </DropdownMenu>
                            </div>
                          )}
                          {selectedMenuId === TRASH_ID && (
                            <div className="flex items-center justify-end gap-2 px-1">
                              <Button
                                type="button"
                                variant="outline"
                                size="sm"
                                className="h-8 px-3 text-xs"
                                onClick={() => {
                                  setTrashActionCipherId(
                                    selectedCipherDetail.id,
                                  );
                                  setTrashActionCipherName(
                                    selectedCipherDetail.name ??
                                      t("vault.page.cipher.untitled"),
                                  );
                                  setIsRestoreDialogOpen(true);
                                }}
                              >
                                <RotateCcw className="size-3.5" />
                                {t("vault.page.actions.restore")}
                              </Button>
                              <Button
                                type="button"
                                variant="destructive"
                                size="sm"
                                className="h-8 px-3 text-xs"
                                onClick={() => {
                                  setTrashActionCipherId(
                                    selectedCipherDetail.id,
                                  );
                                  setTrashActionCipherName(
                                    selectedCipherDetail.name ??
                                      t("vault.page.cipher.untitled"),
                                  );
                                  setIsPermanentDeleteDialogOpen(true);
                                }}
                              >
                                <Trash2 className="size-3.5" />
                                {t("vault.page.actions.permanentDelete")}
                              </Button>
                            </div>
                          )}

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

      <CipherFormDialog
        open={isCipherFormOpen}
        mode={cipherFormMode}
        initialCipher={selectedCipherForEdit}
        folderId={
          selectedMenuId !== ALL_ITEMS_ID &&
          selectedMenuId !== FAVORITES_ID &&
          selectedMenuId !== TRASH_ID &&
          selectedMenuId !== NO_FOLDER_ID
            ? selectedMenuId
            : null
        }
        folders={viewData?.folders ?? []}
        onOpenChange={setIsCipherFormOpen}
        onConfirm={handleCipherFormConfirm}
        isLoading={
          cipherFormMode === "create"
            ? cipherMutations.createCipher.isLoading
            : cipherMutations.updateCipher.isLoading
        }
      />

      <DeleteCipherDialog
        open={isDeleteCipherDialogOpen}
        cipherName={selectedCipherNameForDelete}
        onOpenChange={setIsDeleteCipherDialogOpen}
        onConfirm={handleDeleteCipherConfirm}
        isLoading={cipherMutations.deleteCipher.isLoading}
      />

      {/* Restore confirmation dialog */}
      <Dialog open={isRestoreDialogOpen} onOpenChange={setIsRestoreDialogOpen}>
        <DialogContent className="max-w-md">
          <DialogHeader>
            <DialogTitle className="text-lg font-bold text-slate-900">
              {t("vault.dialogs.restoreCipher.title")}
            </DialogTitle>
            <DialogDescription className="text-sm text-slate-600 pt-2">
              {t("vault.dialogs.restoreCipher.descriptionPrefix")}{" "}
              <span className="font-semibold text-slate-900">
                "{trashActionCipherName}"
              </span>{" "}
              {t("vault.dialogs.restoreCipher.descriptionSuffix")}
            </DialogDescription>
          </DialogHeader>
          <DialogFooter className="gap-2">
            <Button
              variant="outline"
              onClick={() => setIsRestoreDialogOpen(false)}
              disabled={isTrashActionLoading}
              className="h-10"
            >
              {t("common.actions.cancel")}
            </Button>
            <Button
              className="h-10"
              disabled={isTrashActionLoading}
              onClick={async () => {
                setIsTrashActionLoading(true);
                const result = await commands.restoreCipher({
                  cipherId: trashActionCipherId,
                });
                setIsTrashActionLoading(false);
                setIsRestoreDialogOpen(false);
                if (result.status === "error") {
                  toast.error(t("vault.feedback.cipher.restoreError.title"), {
                    description: t(
                      "vault.feedback.cipher.restoreError.description",
                    ),
                  });
                  return;
                }
                toast.success(t("vault.feedback.cipher.restoreSuccess.title"), {
                  description: t(
                    "vault.feedback.cipher.restoreSuccess.description",
                    { name: trashActionCipherName },
                  ),
                });
                void loadVaultData();
              }}
            >
              {isTrashActionLoading ? (
                <>
                  <LoaderCircle className="size-4 animate-spin" />
                  {t("vault.dialogs.restoreCipher.confirming")}
                </>
              ) : (
                t("vault.page.actions.restore")
              )}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Permanent delete confirmation dialog */}
      <Dialog
        open={isPermanentDeleteDialogOpen}
        onOpenChange={setIsPermanentDeleteDialogOpen}
      >
        <DialogContent className="max-w-md">
          <DialogHeader>
            <div className="flex items-center gap-3">
              <div className="flex size-10 items-center justify-center rounded-full bg-red-100">
                <AlertTriangle className="size-5 text-red-600" />
              </div>
              <DialogTitle className="text-lg font-bold text-slate-900">
                {t("vault.dialogs.permanentDeleteCipher.title")}
              </DialogTitle>
            </div>
            <DialogDescription className="text-sm text-slate-600 pt-2">
              {t("vault.dialogs.permanentDeleteCipher.descriptionPrefix")}{" "}
              <span className="font-semibold text-slate-900">
                "{trashActionCipherName}"
              </span>{" "}
              {t("vault.dialogs.permanentDeleteCipher.descriptionSuffix")}
            </DialogDescription>
          </DialogHeader>
          <DialogFooter className="gap-2">
            <Button
              variant="outline"
              onClick={() => setIsPermanentDeleteDialogOpen(false)}
              disabled={isTrashActionLoading}
              className="h-10"
            >
              {t("common.actions.cancel")}
            </Button>
            <Button
              variant="destructive"
              className="h-10"
              disabled={isTrashActionLoading}
              onClick={async () => {
                setIsTrashActionLoading(true);
                const result = await commands.deleteCipher({
                  cipherId: trashActionCipherId,
                });
                setIsTrashActionLoading(false);
                setIsPermanentDeleteDialogOpen(false);
                if (result.status === "error") {
                  toast.error(
                    t("vault.feedback.cipher.permanentDeleteError.title"),
                    {
                      description: t(
                        "vault.feedback.cipher.permanentDeleteError.description",
                      ),
                    },
                  );
                  return;
                }
                toast.success(
                  t("vault.feedback.cipher.permanentDeleteSuccess.title"),
                  {
                    description: t(
                      "vault.feedback.cipher.permanentDeleteSuccess.description",
                      { name: trashActionCipherName },
                    ),
                  },
                );
                void loadVaultData();
              }}
            >
              {isTrashActionLoading ? (
                <>
                  <LoaderCircle className="size-4 animate-spin" />
                  {t("vault.dialogs.permanentDeleteCipher.confirming")}
                </>
              ) : (
                t("vault.page.actions.permanentDelete")
              )}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </main>
  );
}
