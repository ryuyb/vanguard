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
  Plus,
  Search,
  Send,
  Settings,
  Star,
  Trash2,
  UserRound,
  X,
} from "lucide-react";
import { AnimatePresence, motion } from "motion/react";
import { useState } from "react";
import { useTranslation } from "react-i18next";
import { commands } from "@/bindings";
import { TextInput } from "@/components/text-input";
import { Button } from "@/components/ui/button";
import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuTrigger,
} from "@/components/ui/context-menu";
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
import {
  ResizableHandle,
  ResizablePanel,
  ResizablePanelGroup,
} from "@/components/ui/resizable";
import { ScrollArea } from "@/components/ui/scroll-area";
import {
  SendDetailPanel,
  SendDialogs,
  SendListPanel,
  useSendEvents,
  useSendList,
  useSendMutations,
  useSendOperations,
} from "@/features/send";
import {
  ALL_ITEMS_ID,
  CipherDetailPanel,
  CipherRow,
  type CipherSortBy,
  type CipherSortDirection,
  type CipherTypeFilter,
  FAVORITES_ID,
  FolderTreeMenuItem,
  HeaderSearchPanel,
  NO_FOLDER_ID,
  TRASH_ID,
  toTypeFilterLabel,
  useCipherOperations,
  useFolderOperations,
  useTrashOperations,
  VaultDialogs,
} from "@/features/vault";
import { useVaultPageModel } from "@/features/vault/hooks";
import type { VaultPageNavigationTarget } from "@/features/vault/hooks/use-vault-page-model";
import { toast } from "@/lib/toast";

type VaultPageProps = {
  navigateTo: (to: VaultPageNavigationTarget) => Promise<void>;
};

function CipherRowObserver({ children }: { children: React.ReactNode }) {
  return <div>{children}</div>;
}

export function VaultPage({ navigateTo }: VaultPageProps) {
  const { t } = useTranslation();
  const [isSettingsDialogOpen, setIsSettingsDialogOpen] = useState(false);

  // 使用 operations hooks 替代分散的状态和 handler
  const folderOps = useFolderOperations();
  const cipherOps = useCipherOperations();
  const trashOps = useTrashOperations({
    onSuccess: () => void loadVaultData(),
  });
  const sendOps = useSendOperations();
  const sendMutations = useSendMutations();

  // Send events handling
  useSendEvents({
    onSendCreated: () => void reloadSends(),
    onSendUpdated: () => void reloadSends(),
    onSendDeleted: () => {
      void reloadSends();
      sendOps.setSelectedSendId(null);
    },
  });

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
    headerSearchResults,
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
    isVaultUnlockedSessionExpired,
  } = useVaultPageModel({ navigateTo });

  const {
    filteredSends,
    sendCount,
    isLoading: isSendLoading,
    sendTypeFilter,
    setSendTypeFilter,
    reload: reloadSends,
  } = useSendList(pageState === "ready");

  // 计算 cipherFolderId（当前选中的文件夹 ID）
  const cipherFolderId =
    selectedMenuId !== ALL_ITEMS_ID &&
    selectedMenuId !== FAVORITES_ID &&
    selectedMenuId !== TRASH_ID &&
    selectedMenuId !== NO_FOLDER_ID &&
    selectedMenuId !== "send"
      ? selectedMenuId
      : null;

  // 处理删除文件夹后切换菜单
  const handleDeleteFolderConfirm = async () => {
    await folderOps.handleDeleteDialogConfirm();
    if (
      folderOps.selectedFolderId &&
      selectedMenuId === folderOps.selectedFolderId
    ) {
      setSelectedMenuId(ALL_ITEMS_ID);
    }
  };

  // Remove password handler for SendDialogs
  const handleRemoveSendPassword = async (sendId: string) => {
    await sendMutations.removeSendPassword.mutateAsync(sendId);
    toast.success(t("send.feedback.removePasswordSuccess"));
  };

  return (
    <main className="flex h-dvh flex-col bg-gradient-to-br from-slate-50 via-white to-slate-100">
      <header
        data-tauri-drag-region
        className="relative z-50 w-full border-b border-slate-200/60 bg-white/80 px-4 py-2 shadow-sm backdrop-blur-md md:px-8 md:py-2.5"
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
              <TextInput
                ref={searchInputRef}
                type="search"
                placeholder={t("vault.page.search.placeholder")}
                value={headerSearchQuery}
                onChange={(event) => setHeaderSearchQuery(event.target.value)}
                className="h-9 pl-9 text-sm border-slate-200 bg-slate-50/50 focus:bg-white transition-colors"
              />
              <AnimatePresence>
                {headerSearchQuery.trim() && headerSearchResults.length > 0 && (
                  <HeaderSearchPanel
                    key="header-search-panel"
                    results={headerSearchResults}
                    onSelect={(cipherId) => {
                      void loadCipherDetail(cipherId);
                    }}
                    onClose={() => {
                      setHeaderSearchQuery("");
                    }}
                  />
                )}
              </AnimatePresence>
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
        {/* Session expired warning - allow local operations but warn user */}
        {isVaultUnlockedSessionExpired && (
          <div className="rounded-xl border border-amber-200 bg-amber-50/80 px-4 py-3 text-sm text-amber-900 shadow-sm">
            <div className="flex items-center gap-2">
              <AlertTriangle className="h-4 w-4 text-amber-600" />
              <span className="font-medium">
                {t("vault.page.states.sessionExpired.title")}
              </span>
            </div>
            <p className="mt-1 text-amber-800">
              {t("vault.page.states.sessionExpired.description")}
            </p>
          </div>
        )}

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

                    <button
                      type="button"
                      onClick={() => {
                        setSelectedMenuId("send");
                        sendOps.setSelectedSendId(null);
                        void reloadSends();
                      }}
                      className={[
                        "w-full rounded-lg px-3 py-2 text-left text-sm font-medium transition-all",
                        selectedMenuId === "send"
                          ? "bg-blue-50 text-blue-700 shadow-sm"
                          : "text-slate-700 hover:bg-slate-50",
                      ].join(" ")}
                    >
                      <div className="flex items-center justify-between gap-2">
                        <span className="inline-flex items-center gap-2.5">
                          <Send
                            className={[
                              "size-4",
                              selectedMenuId === "send"
                                ? "text-blue-600"
                                : "text-slate-400",
                            ].join(" ")}
                          />
                          {t("vault.page.menus.send")}
                        </span>
                        <span
                          className={[
                            "text-xs font-semibold",
                            selectedMenuId === "send"
                              ? "text-blue-600"
                              : "text-slate-400",
                          ].join(" ")}
                        >
                          {sendCount}
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
                        onClick={folderOps.handleCreateFolder}
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
                          onRenameFolder={folderOps.handleRenameFolder}
                          onDeleteFolder={folderOps.handleDeleteFolder}
                          onCreateSubFolder={folderOps.handleCreateSubFolder}
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
              {selectedMenuId === "send" ? (
                <SendListPanel
                  sends={filteredSends}
                  isLoading={isSendLoading}
                  selectedSendId={sendOps.selectedSendId}
                  sendTypeFilter={sendTypeFilter}
                  onSelectSend={sendOps.setSelectedSendId}
                  onCreateSend={sendOps.handleCreateSend}
                  onFilterChange={setSendTypeFilter}
                  baseUrl={userBaseUrl}
                  onEdit={(send) => sendOps.handleEditSend(send)}
                  onDelete={sendOps.handleDeleteSend}
                />
              ) : (
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
                            <TextInput
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
                        onClick={cipherOps.handleCreateCipher}
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

                  <ScrollArea className="min-h-0 flex-1 overflow-hidden [&>div>div]:!block">
                    <div className="space-y-1.5 px-3 py-2 min-w-0 w-full">
                      {filteredCiphers.map(
                        (cipher: (typeof filteredCiphers)[number]) => (
                          <CipherRowObserver key={cipher.id}>
                            <ContextMenu>
                              <ContextMenuTrigger asChild>
                                <div className="min-w-0">
                                  <CipherRow
                                    cipher={cipher}
                                    onClick={() => {
                                      void loadCipherDetail(cipher.id);
                                    }}
                                    selected={cipher.id === selectedCipherId}
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
                                        cipherOps.handleEditCipher(
                                          result.data.cipher,
                                        );
                                      }}
                                    >
                                      <Edit2 className="size-4" />
                                      {t("vault.page.cipher.contextMenu.edit")}
                                    </ContextMenuItem>
                                    <ContextMenuItem
                                      onSelect={async () => {
                                        const result =
                                          await commands.vaultGetCipherDetail({
                                            cipherId: cipher.id,
                                          });
                                        if (result.status === "error") return;
                                        await cipherOps.handleCloneCipher(
                                          result.data.cipher,
                                        );
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
              )}
            </ResizablePanel>

            <ResizableHandle withHandle className="bg-slate-200" />

            <ResizablePanel defaultSize={40} minSize={26}>
              <section className="h-full min-h-0 min-w-0 overflow-x-hidden bg-white">
                <ScrollArea className="h-full min-w-0 [&>[data-slot=scroll-area-viewport]]:min-w-0 [&>[data-slot=scroll-area-viewport]]:overflow-x-hidden [&>[data-slot=scroll-area-viewport]>div]:!block [&>[data-slot=scroll-area-viewport]>div]:h-full [&>[data-slot=scroll-area-viewport]>div]:min-w-0 [&>[data-slot=scroll-area-viewport]>div]:w-full">
                  <div className="flex h-full min-w-0 w-full flex-col p-3">
                    {selectedMenuId === "send" ? (
                      <SendDetailPanel
                        sendId={sendOps.selectedSendId}
                        sends={filteredSends}
                        baseUrl={userBaseUrl}
                        onEdit={(send) => sendOps.handleEditSend(send)}
                        onDelete={sendOps.handleDeleteSend}
                      />
                    ) : (
                      <>
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
                            <div className="min-h-0 min-w-0 w-full flex-1 overflow-x-hidden">
                              <CipherDetailPanel
                                key={selectedCipherDetail.id}
                                cipher={selectedCipherDetail}
                                mode={
                                  selectedMenuId === TRASH_ID
                                    ? "trash"
                                    : "normal"
                                }
                                onEdit={() =>
                                  cipherOps.handleEditCipher(
                                    selectedCipherDetail,
                                  )
                                }
                                onDelete={() =>
                                  cipherOps.handleDeleteCipher(
                                    selectedCipherDetail.id,
                                    selectedCipherDetail.name ??
                                      t("vault.page.cipher.untitled"),
                                  )
                                }
                                onRestore={() =>
                                  trashOps.handleRestoreCipher(
                                    selectedCipherDetail.id,
                                    selectedCipherDetail.name ??
                                      t("vault.page.cipher.untitled"),
                                  )
                                }
                                onPermanentDelete={() =>
                                  trashOps.handlePermanentDeleteCipher(
                                    selectedCipherDetail.id,
                                    selectedCipherDetail.name ??
                                      t("vault.page.cipher.untitled"),
                                  )
                                }
                                isActionLoading={trashOps.isTrashActionLoading}
                              />
                            </div>
                          )}
                      </>
                    )}
                  </div>
                </ScrollArea>
              </section>
            </ResizablePanel>
          </ResizablePanelGroup>
        )}
      </section>

      {/* 使用 VaultDialogs 容器组件替换所有内联 Vault 对话框 */}
      <VaultDialogs
        // Settings
        isSettingsDialogOpen={isSettingsDialogOpen}
        onSettingsDialogOpenChange={setIsSettingsDialogOpen}
        // Folder operations
        folderDialogMode={folderOps.folderDialogMode}
        isFolderDialogOpen={folderOps.isFolderDialogOpen}
        isDeleteFolderDialogOpen={folderOps.isDeleteDialogOpen}
        selectedFolderName={folderOps.selectedFolderName}
        parentFolderName={folderOps.parentFolderName}
        onFolderDialogOpenChange={(open) => {
          if (!open) folderOps.closeFolderDialog();
        }}
        onDeleteFolderDialogOpenChange={(open) => {
          if (!open) folderOps.closeDeleteDialog();
        }}
        onFolderDialogConfirm={(name) =>
          void folderOps.handleFolderDialogConfirm(name)
        }
        onDeleteFolderDialogConfirm={() => void handleDeleteFolderConfirm()}
        isFolderLoading={
          folderOps.folderDialogMode === "create"
            ? folderOps.isCreating
            : folderOps.isRenaming || folderOps.isDeleting
        }
        // Cipher operations
        cipherFormMode={cipherOps.cipherFormMode}
        isCipherFormOpen={cipherOps.isCipherFormOpen}
        isDeleteCipherDialogOpen={cipherOps.isDeleteCipherDialogOpen}
        selectedCipherForEdit={cipherOps.selectedCipherForEdit}
        selectedCipherNameForDelete={cipherOps.selectedCipherNameForDelete}
        cipherFolderId={cipherFolderId}
        folders={viewData?.folders ?? []}
        onCipherFormOpenChange={(open) => {
          if (!open) cipherOps.closeCipherFormDialog();
        }}
        onDeleteCipherDialogOpenChange={(open) => {
          if (!open) cipherOps.closeDeleteCipherDialog();
        }}
        onCipherFormConfirm={(cipher) =>
          void cipherOps.handleCipherFormConfirm(cipher)
        }
        onDeleteCipherDialogConfirm={() =>
          void cipherOps.handleDeleteCipherConfirm()
        }
        isCipherFormLoading={
          cipherOps.cipherFormMode === "create"
            ? cipherOps.isCreating
            : cipherOps.isUpdating
        }
        isDeleteCipherLoading={cipherOps.isDeleting}
        // Trash operations
        isRestoreDialogOpen={trashOps.isRestoreDialogOpen}
        isPermanentDeleteDialogOpen={trashOps.isPermanentDeleteDialogOpen}
        trashActionCipherName={trashOps.trashActionCipherName}
        onRestoreDialogOpenChange={(open) => {
          if (!open) trashOps.closeRestoreDialog();
        }}
        onPermanentDeleteDialogOpenChange={(open) => {
          if (!open) trashOps.closePermanentDeleteDialog();
        }}
        onRestoreDialogConfirm={() => void trashOps.handleRestoreConfirm()}
        onPermanentDeleteDialogConfirm={() =>
          void trashOps.handlePermanentDeleteConfirm()
        }
        isTrashActionLoading={trashOps.isTrashActionLoading}
      />

      {/* 使用 SendDialogs 容器组件替换所有内联 Send 对话框 */}
      <SendDialogs
        isSendFormOpen={sendOps.isSendFormOpen}
        sendFormMode={sendOps.sendFormMode}
        selectedSendForEdit={sendOps.selectedSendForEdit}
        isDeleteSendDialogOpen={sendOps.isDeleteSendDialogOpen}
        selectedSendNameForDelete={sendOps.selectedSendNameForDelete}
        onSendFormOpenChange={(open) => {
          if (!open) sendOps.closeSendFormDialog();
        }}
        onDeleteSendOpenChange={(open) => {
          if (!open) sendOps.closeDeleteSendDialog();
        }}
        onSendFormConfirm={async (send, fileData) => {
          await sendOps.handleSendFormConfirm(send, fileData);
        }}
        onDeleteSendConfirm={async () => {
          await sendOps.handleDeleteSendConfirm();
        }}
        onRemovePassword={async (sendId) => {
          await handleRemoveSendPassword(sendId);
        }}
        isSendFormLoading={sendOps.isCreating || sendOps.isUpdating}
        isDeleteSendLoading={sendOps.isDeleting}
      />
    </main>
  );
}
