import { emitTo } from "@tauri-apps/api/event";
import { getCurrentWindow, Window } from "@tauri-apps/api/window";
import { error as logError } from "@tauri-apps/plugin-log";
import { Command as CommandPrimitive } from "cmdk";
import { SearchIcon } from "lucide-react";
import type { KeyboardEvent } from "react";
import { StrictMode, useCallback, useEffect, useMemo, useState } from "react";
import ReactDOM from "react-dom/client";
import { commands, type VaultCipherItemDto } from "@/bindings";
import { Badge } from "@/components/ui/badge";
import { Card, CardFooter } from "@/components/ui/card";
import { Command, CommandItem, CommandList } from "@/components/ui/command";
import { InputGroup, InputGroupAddon } from "@/components/ui/input-group";
import { Kbd, KbdGroup } from "@/components/ui/kbd";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Separator } from "@/components/ui/separator";
import { Skeleton } from "@/components/ui/skeleton";
import "@/main.css";
import "./spotlight.css";

type SpotlightItem = {
  id: string;
  kind: "cipher";
  title: string;
  subtitle: string;
  badge: string;
  cipherId: string;
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

function toCipherItem(cipher: VaultCipherItemDto): SpotlightItem {
  return {
    id: `cipher-${cipher.id}`,
    kind: "cipher",
    cipherId: cipher.id,
    title: cipher.name?.trim() ? cipher.name : "Untitled Cipher",
    subtitle: "Vault item",
    badge: "Cipher",
  };
}

function SpotlightApp() {
  const [query, setQuery] = useState("");
  const [vaultItems, setVaultItems] = useState<SpotlightItem[]>([]);
  const [isLoadingVault, setIsLoadingVault] = useState(false);

  const hideSpotlight = useCallback(async () => {
    try {
      await getCurrentWindow().hide();
    } catch (error) {
      logClientError("Failed to hide spotlight", error);
    }
  }, []);

  const focusMainWindow = useCallback(async () => {
    const mainWindow = await Window.getByLabel("main");
    if (!mainWindow) {
      return;
    }
    await mainWindow.show();
    await mainWindow.setFocus();
  }, []);

  const loadVaultItems = useCallback(async () => {
    setIsLoadingVault(true);

    try {
      const viewData = await commands.vaultGetViewData({
        page: 1,
        pageSize: 200,
      });
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

  useEffect(() => {
    void loadVaultItems();
  }, [loadVaultItems]);

  const visibleItems = useMemo(() => {
    const normalized = query.trim().toLowerCase();
    if (!normalized) {
      return vaultItems;
    }

    return vaultItems.filter((item) => {
      const haystack = `${item.title} ${item.subtitle}`.toLowerCase();
      return haystack.includes(normalized);
    });
  }, [query, vaultItems]);

  const executeItem = useCallback(
    async (_item: SpotlightItem) => {
      try {
        await emitTo("main", "spotlight:navigate", { to: "/vault" });
        await focusMainWindow();
        await hideSpotlight();
      } catch (error) {
        logClientError("Action failed", error);
      }
    },
    [focusMainWindow, hideSpotlight],
  );

  const onCommandInputKeyDown = useCallback(
    (event: KeyboardEvent<HTMLInputElement>) => {
      if (event.key === "Escape") {
        event.preventDefault();
        void hideSpotlight();
      }
    },
    [hideSpotlight],
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
        <Command
          className="spotlight-command"
          value={query}
          onValueChange={setQuery}
          shouldFilter={false}
          loop
        >
          <div className="spotlight-search">
            <InputGroup className="spotlight-search-group">
              <CommandPrimitive.Input
                id="spotlight-search-input"
                className="spotlight-input"
                autoFocus
                spellCheck={false}
                autoCorrect="off"
                autoCapitalize="off"
                placeholder="Search passwords, commands, or pages"
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
          <ScrollArea className="spotlight-results">
            {isLoadingVault ? (
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
            ) : (
              <CommandList className="spotlight-command-list">
                {visibleItems.map((item) => (
                  <CommandItem
                    key={item.id}
                    value={item.id}
                    className="spotlight-item"
                    onSelect={() => {
                      void executeItem(item);
                    }}
                  >
                    <div>
                      <p className="spotlight-item-title">{item.title}</p>
                      <p className="spotlight-item-subtitle">{item.subtitle}</p>
                    </div>
                    <Badge variant="secondary" className="spotlight-item-badge">
                      {item.badge}
                    </Badge>
                  </CommandItem>
                ))}
              </CommandList>
            )}
          </ScrollArea>
        </Command>

        <Separator className="spotlight-meta-separator" />

        <CardFooter className="spotlight-meta">
          <span className="spotlight-meta-item">
            <KbdGroup>
              <Kbd>↑</Kbd>
              <Kbd>↓</Kbd>
            </KbdGroup>
            Navigate
          </span>
          <span className="spotlight-meta-item">
            <Kbd>Enter</Kbd>
            Run
          </span>
          <span className="spotlight-meta-item">
            <Kbd>Esc</Kbd>
            Close
          </span>
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
