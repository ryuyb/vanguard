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
import { useCallback } from "react";
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
  TRASH_ID,
  toTypeFilterLabel,
  useVaultPageModel,
} from "@/features/vault";
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
  const navigateTo = useCallback(
    async (to: "/" | "/unlock" | "/vault") => {
      await navigate({ to });
    },
    [navigate],
  );

  const {
    avatarText,
    cipherDetailError,
    cipherSearchQuery,
    errorText,
    expandedNodeKeys,
    filteredCiphers,
    folderCipherCount,
    folderTree,
    headerSearchQuery,
    inlineSearchInputRef,
    isCipherDetailLoading,
    isHeaderActionBusy,
    isInlineSearchOpen,
    isLocking,
    isLoggingOut,
    loadCipherDetail,
    lockLabel,
    logoutLabel,
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
    typeFilter,
    userBaseUrl,
    userEmail,
    viewData,
  } = useVaultPageModel({ navigateTo });

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
