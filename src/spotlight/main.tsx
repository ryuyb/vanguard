import { getCurrentWindow } from "@tauri-apps/api/window";
import { error as logError } from "@tauri-apps/plugin-log";
import { Command as CommandPrimitive } from "cmdk";
import { SearchIcon } from "lucide-react";
import type { KeyboardEvent } from "react";
import { StrictMode, useCallback, useEffect, useMemo, useState } from "react";
import ReactDOM from "react-dom/client";
import { commands, type VaultCipherItemDto } from "@/bindings";
import { Card, CardFooter } from "@/components/ui/card";
import { Command, CommandItem } from "@/components/ui/command";
import { InputGroup, InputGroupAddon } from "@/components/ui/input-group";
import { Kbd, KbdGroup } from "@/components/ui/kbd";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Separator } from "@/components/ui/separator";
import { Skeleton } from "@/components/ui/skeleton";
import { resolveSessionRoute } from "@/lib/route-session";
import "@/main.css";
import "./spotlight.css";

type SpotlightItem = {
  id: string;
  title: string;
  subtitle: string;
  searchText: string;
};

function errorToText(error: unknown) {
  if (typeof error === "string") {
    return error;
  }
  if (error instanceof Error) {
    return error.message;
  }
  return "Unknown error";
}

function logClientError(context: string, error: unknown) {
  void logError(`[spotlight] ${context}: ${errorToText(error)}`);
}

const DETAIL_ACTIONS = [
  { label: "复制 用户名", shortcut: ["⌘", "C"] },
  { label: "复制 密码", shortcut: ["⌘", "⇧", "C"] },
] as const;

function toCipherItem(cipher: VaultCipherItemDto): SpotlightItem {
  const rawName = cipher.name?.trim() ?? "";
  const rawUsername = cipher.username?.trim() ?? "";
  const title = rawName || "Untitled Cipher";
  const subtitle = rawUsername || "Vault item";
  return {
    id: `cipher-${cipher.id}`,
    title,
    subtitle,
    searchText: `${rawName} ${rawUsername}`.toLowerCase(),
  };
}

function SpotlightApp() {
  const [query, setQuery] = useState("");
  const [vaultItems, setVaultItems] = useState<SpotlightItem[]>([]);
  const [isLoadingVault, setIsLoadingVault] = useState(false);
  const [detailItemId, setDetailItemId] = useState<string | null>(null);
  const [detailActionIndex, setDetailActionIndex] = useState(0);

  const hideSpotlight = useCallback(async () => {
    try {
      await getCurrentWindow().hide();
    } catch (error) {
      logClientError("Failed to hide spotlight", error);
    }
  }, []);

  const openMainWindow = useCallback(async () => {
    try {
      const result = await commands.desktopOpenMainWindow();
      if (result.status === "error") {
        logClientError(
          "Failed to open main window via desktop command",
          result.error,
        );
      }
    } catch (error) {
      logClientError("Failed to open main window via desktop command", error);
    } finally {
      await hideSpotlight();
    }
  }, [hideSpotlight]);

  const ensureSpotlightSession = useCallback(async () => {
    try {
      const target = await resolveSessionRoute();
      if (target === "/vault") {
        return true;
      }

      await openMainWindow();
      return false;
    } catch (error) {
      logClientError("Failed to resolve spotlight session route", error);
      await hideSpotlight();
      return false;
    }
  }, [hideSpotlight, openMainWindow]);

  const loadVaultItems = useCallback(async () => {
    setIsLoadingVault(true);

    try {
      const viewData = await commands.vaultGetViewData();
      if (viewData.status === "error") {
        logClientError("Failed to load vault data", viewData.error);
        setVaultItems([]);
        return;
      }

      const ciphers = viewData.data.ciphers.map(toCipherItem);
      setVaultItems(ciphers);
    } catch (error) {
      logClientError("Failed to load vault data", error);
      setVaultItems([]);
    } finally {
      setIsLoadingVault(false);
    }
  }, []);

  const refreshSpotlightState = useCallback(async () => {
    const canUseSpotlight = await ensureSpotlightSession();
    if (!canUseSpotlight) {
      return;
    }
    await loadVaultItems();
  }, [ensureSpotlightSession, loadVaultItems]);

  useEffect(() => {
    void refreshSpotlightState();
  }, [refreshSpotlightState]);

  useEffect(() => {
    let unlisten: (() => void) | null = null;
    let disposed = false;

    void getCurrentWindow()
      .onFocusChanged(({ payload: focused }) => {
        if (!focused) {
          return;
        }
        void refreshSpotlightState();
      })
      .then((unsubscribe) => {
        if (disposed) {
          unsubscribe();
          return;
        }
        unlisten = unsubscribe;
      })
      .catch((error) => {
        logClientError("Failed to subscribe spotlight focus events", error);
      });

    return () => {
      disposed = true;
      unlisten?.();
    };
  }, [refreshSpotlightState]);

  useEffect(() => {
    const handleWindowFocus = () => {
      void refreshSpotlightState();
    };
    const handleVisibilityChange = () => {
      if (document.visibilityState !== "visible") {
        return;
      }
      void refreshSpotlightState();
    };

    window.addEventListener("focus", handleWindowFocus);
    window.addEventListener("pageshow", handleWindowFocus);
    document.addEventListener("visibilitychange", handleVisibilityChange);
    return () => {
      window.removeEventListener("focus", handleWindowFocus);
      window.removeEventListener("pageshow", handleWindowFocus);
      document.removeEventListener("visibilitychange", handleVisibilityChange);
    };
  }, [refreshSpotlightState]);

  const normalizedQuery = query.trim().toLowerCase();
  const hasQuery = normalizedQuery.length > 0;

  const visibleItems = useMemo(() => {
    if (!hasQuery) {
      return [];
    }

    return vaultItems.filter((item) =>
      item.searchText.includes(normalizedQuery),
    );
  }, [hasQuery, normalizedQuery, vaultItems]);
  const detailItem = useMemo(() => {
    if (!detailItemId) {
      return null;
    }
    return visibleItems.find((item) => item.id === detailItemId) ?? null;
  }, [detailItemId, visibleItems]);
  const hasVisibleResults = visibleItems.length > 0;
  const shouldShowResults =
    (isLoadingVault && hasQuery) || visibleItems.length > 0;

  useEffect(() => {
    if (!hasVisibleResults) {
      setDetailItemId(null);
      return;
    }
    if (!detailItemId) {
      return;
    }
    const isVisible = visibleItems.some((item) => item.id === detailItemId);
    if (!isVisible) {
      setDetailItemId(null);
    }
  }, [detailItemId, hasVisibleResults, visibleItems]);

  useEffect(() => {
    if (!detailItem) {
      setDetailActionIndex(0);
    }
  }, [detailItem]);

  const resolveSelectedItemId = useCallback(() => {
    const selectedElement = document.querySelector<HTMLElement>(
      "#spotlight-card .spotlight-item[data-selected='true']",
    );
    const selectedValue = selectedElement?.getAttribute("data-value");
    if (
      selectedValue &&
      visibleItems.some((item) => item.id === selectedValue)
    ) {
      return selectedValue;
    }
    return visibleItems[0]?.id ?? null;
  }, [visibleItems]);

  const onCommandInputKeyDown = useCallback(
    (event: KeyboardEvent<HTMLInputElement>) => {
      if (
        detailItem &&
        (event.key === "ArrowDown" || event.key === "ArrowUp")
      ) {
        event.preventDefault();
        setDetailActionIndex((current) => {
          if (event.key === "ArrowDown") {
            return (current + 1) % DETAIL_ACTIONS.length;
          }
          return (current - 1 + DETAIL_ACTIONS.length) % DETAIL_ACTIONS.length;
        });
        return;
      }

      if (event.key === "ArrowRight" && hasVisibleResults && !detailItem) {
        const inputElement = event.currentTarget;
        const inputValueLength = inputElement.value.length;
        const isCaretAtEnd =
          inputElement.selectionStart === inputValueLength &&
          inputElement.selectionEnd === inputValueLength;
        if (!isCaretAtEnd) {
          return;
        }
        event.preventDefault();
        const selectedItemId = resolveSelectedItemId();
        if (selectedItemId) {
          setDetailItemId(selectedItemId);
        }
        return;
      }

      if (event.key === "ArrowLeft" && detailItem) {
        event.preventDefault();
        setDetailItemId(null);
        return;
      }

      if (event.key === "Escape") {
        event.preventDefault();
        if (detailItem) {
          setDetailItemId(null);
          return;
        }
        if (query.trim().length > 0) {
          setQuery("");
          return;
        }
        void hideSpotlight();
      }
    },
    [
      detailItem,
      hasVisibleResults,
      hideSpotlight,
      query,
      resolveSelectedItemId,
    ],
  );

  useEffect(() => {
    const onDocumentMouseDown = (event: globalThis.MouseEvent) => {
      const target = event.target;
      if (!(target instanceof Node)) {
        return;
      }
      const cardElement = document.getElementById("spotlight-card");
      if (cardElement?.contains(target)) {
        return;
      }
      void hideSpotlight();
    };

    document.addEventListener("mousedown", onDocumentMouseDown, true);
    return () => {
      document.removeEventListener("mousedown", onDocumentMouseDown, true);
    };
  }, [hideSpotlight]);

  return (
    <main className="spotlight-shell">
      <Card id="spotlight-card" className="spotlight-card">
        <Command className="spotlight-command" shouldFilter={false} loop>
          <div className="spotlight-search">
            <InputGroup className="spotlight-search-group">
              <CommandPrimitive.Input
                id="spotlight-search-input"
                className="spotlight-input"
                value={query}
                onValueChange={setQuery}
                autoFocus
                spellCheck={false}
                autoCorrect="off"
                autoCapitalize="off"
                placeholder="Search vault ciphers by keyword"
                aria-label="Search"
                onKeyDown={onCommandInputKeyDown}
              />
              <InputGroupAddon
                align="inline-end"
                className="spotlight-search-addon"
              >
                <SearchIcon className="size-4 shrink-0" />
              </InputGroupAddon>
            </InputGroup>
          </div>
          {shouldShowResults ? (
            <ScrollArea className="spotlight-results">
              {isLoadingVault && hasQuery ? (
                <div className="spotlight-skeleton-list" aria-hidden>
                  {Array.from({ length: 6 }).map((_, index) => (
                    <div
                      key={`skeleton-${index}`}
                      className="spotlight-skeleton-item"
                    >
                      <Skeleton className="h-4 w-3/5" />
                      <Skeleton className="h-3 w-2/5" />
                    </div>
                  ))}
                </div>
              ) : detailItem ? (
                <section
                  className="spotlight-detail"
                  aria-label="Cipher detail"
                >
                  <p className="spotlight-detail-title">{detailItem.title}</p>
                  <p className="spotlight-detail-subtitle">
                    {detailItem.subtitle}
                  </p>
                  <div className="spotlight-detail-actions" role="listbox">
                    {DETAIL_ACTIONS.map((action, index) => (
                      <div
                        key={action.label}
                        role="option"
                        tabIndex={-1}
                        aria-selected={detailActionIndex === index}
                        className={`spotlight-detail-action${detailActionIndex === index ? " is-selected" : ""}`}
                      >
                        <span>{action.label}</span>
                        <KbdGroup className="spotlight-detail-action-shortcut">
                          {action.shortcut.map((shortcutKey) => (
                            <Kbd key={`${action.label}-${shortcutKey}`}>
                              {shortcutKey}
                            </Kbd>
                          ))}
                        </KbdGroup>
                      </div>
                    ))}
                  </div>
                </section>
              ) : (
                <CommandPrimitive.List className="spotlight-command-list">
                  {visibleItems.map((item) => (
                    <CommandItem
                      key={item.id}
                      value={item.id}
                      className="spotlight-item"
                    >
                      <div>
                        <p className="spotlight-item-title">{item.title}</p>
                        <p className="spotlight-item-subtitle">
                          {item.subtitle}
                        </p>
                      </div>
                    </CommandItem>
                  ))}
                </CommandPrimitive.List>
              )}
            </ScrollArea>
          ) : null}
        </Command>

        <Separator className="spotlight-meta-separator" />

        <CardFooter className="spotlight-meta">
          {hasVisibleResults ? (
            <>
              {detailItem ? null : (
                <>
                  <span className="spotlight-meta-item">
                    <KbdGroup>
                      <Kbd>⌘</Kbd>
                      <Kbd>C</Kbd>
                    </KbdGroup>
                    复制 用户名
                  </span>
                  <span className="spotlight-meta-item">
                    <KbdGroup>
                      <Kbd>⌘</Kbd>
                      <Kbd>⇧</Kbd>
                      <Kbd>C</Kbd>
                    </KbdGroup>
                    复制 密码
                  </span>
                </>
              )}
              <span className="spotlight-meta-item">
                <Kbd>{detailItem ? "←" : "→"}</Kbd>
                {detailItem ? "返回结果" : "更多操作"}
              </span>
              {detailItem ? (
                <span className="spotlight-meta-item">
                  <KbdGroup>
                    <Kbd>↑</Kbd>
                    <Kbd>↓</Kbd>
                  </KbdGroup>
                  选择
                </span>
              ) : null}
            </>
          ) : (
            <>
              <span className="spotlight-meta-item">
                <KbdGroup>
                  <Kbd>⇧</Kbd>
                  <Kbd>⌃</Kbd>
                  <Kbd>Space</Kbd>
                </KbdGroup>
                Open quick access
              </span>
              <span className="spotlight-meta-item">
                <Kbd>Esc</Kbd>
                Close
              </span>
            </>
          )}
        </CardFooter>
      </Card>
    </main>
  );
}

ReactDOM.createRoot(
  document.getElementById("spotlight-root") as HTMLElement,
).render(
  <StrictMode>
    <SpotlightApp />
  </StrictMode>,
);
