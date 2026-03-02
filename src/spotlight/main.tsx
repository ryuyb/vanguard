import { emitTo } from "@tauri-apps/api/event";
import { Window, getCurrentWindow } from "@tauri-apps/api/window";
import { StrictMode } from "react";
import type { KeyboardEvent } from "react";
import { useCallback, useEffect, useMemo, useState } from "react";
import ReactDOM from "react-dom/client";
import { commands, type VaultCipherItemDto } from "@/bindings";
import "./spotlight.css";

type SpotlightRoute = "/" | "/unlock" | "/vault";

type SpotlightItem =
  | {
      id: string;
      kind: "route";
      title: string;
      subtitle: string;
      badge: string;
      route: SpotlightRoute;
    }
  | {
      id: string;
      kind: "sync";
      title: string;
      subtitle: string;
      badge: string;
    }
  | {
      id: string;
      kind: "cipher";
      title: string;
      subtitle: string;
      badge: string;
      cipherId: string;
    };

const QUICK_ITEMS: SpotlightItem[] = [
  {
    id: "route-vault",
    kind: "route",
    title: "Open Vault",
    subtitle: "Navigate to vault page",
    badge: "Route",
    route: "/vault",
  },
  {
    id: "route-unlock",
    kind: "route",
    title: "Unlock Vault",
    subtitle: "Navigate to unlock page",
    badge: "Route",
    route: "/unlock",
  },
  {
    id: "route-login",
    kind: "route",
    title: "Open Login",
    subtitle: "Navigate to login page",
    badge: "Route",
    route: "/",
  },
  {
    id: "action-sync",
    kind: "sync",
    title: "Sync Now",
    subtitle: "Run vault_sync_now command",
    badge: "Action",
  },
];

function errorToText(error: unknown) {
  if (typeof error === "string") {
    return error;
  }
  if (error instanceof Error) {
    return error.message;
  }
  return "Unknown error";
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
  const [statusText, setStatusText] = useState("");
  const [selectedIndex, setSelectedIndex] = useState(0);

  const hideSpotlight = useCallback(async () => {
    try {
      await getCurrentWindow().hide();
    } catch (error) {
      setStatusText(`Failed to hide spotlight: ${errorToText(error)}`);
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
    setStatusText("");

    try {
      const restore = await commands.authRestoreState({});
      if (restore.status === "error") {
        setStatusText(`Failed to restore state: ${errorToText(restore.error)}`);
        setVaultItems([]);
        return;
      }

      if (restore.data.status === "needsLogin") {
        setStatusText("Not logged in. Login to search vault items.");
        setVaultItems([]);
        return;
      }

      if (restore.data.status === "locked") {
        setStatusText("Vault is locked. Unlock to search vault items.");
        setVaultItems([]);
        return;
      }

      const viewData = await commands.vaultGetViewData({ page: 1, pageSize: 200 });
      if (viewData.status === "error") {
        setStatusText(`Failed to load vault data: ${errorToText(viewData.error)}`);
        setVaultItems([]);
        return;
      }

      const ciphers = viewData.data.ciphers.map(toCipherItem);
      setVaultItems(ciphers);
      if (ciphers.length === 0) {
        setStatusText("No vault items found.");
      }
    } catch (error) {
      setStatusText(`Failed to load vault data: ${errorToText(error)}`);
      setVaultItems([]);
    } finally {
      setIsLoadingVault(false);
    }
  }, []);

  useEffect(() => {
    void loadVaultItems();
  }, [loadVaultItems]);

  const searchableItems = useMemo(
    () => [...vaultItems, ...QUICK_ITEMS],
    [vaultItems],
  );

  const visibleItems = useMemo(() => {
    const normalized = query.trim().toLowerCase();
    if (!normalized) {
      return vaultItems;
    }

    return searchableItems.filter((item) => {
      const haystack = `${item.title} ${item.subtitle}`.toLowerCase();
      return haystack.includes(normalized);
    });
  }, [query, searchableItems, vaultItems]);

  useEffect(() => {
    if (visibleItems.length === 0) {
      setSelectedIndex(-1);
      return;
    }
    setSelectedIndex(0);
  }, [visibleItems]);

  const executeItem = useCallback(
    async (item: SpotlightItem) => {
      setStatusText("");

      try {
        if (item.kind === "route") {
          await emitTo("main", "spotlight:navigate", { to: item.route });
          await focusMainWindow();
          await hideSpotlight();
          return;
        }

        if (item.kind === "sync") {
          setStatusText("Syncing...");
          const syncResult = await commands.vaultSyncNow({ excludeDomains: false });
          if (syncResult.status === "error") {
            setStatusText(`Sync failed: ${errorToText(syncResult.error)}`);
            return;
          }

          setStatusText("Sync succeeded.");
          await hideSpotlight();
          return;
        }

        await emitTo("main", "spotlight:navigate", { to: "/vault" });
        await focusMainWindow();
        await hideSpotlight();
      } catch (error) {
        setStatusText(`Action failed: ${errorToText(error)}`);
      }
    },
    [focusMainWindow, hideSpotlight],
  );

  const onInputKeyDown = useCallback(
    (event: KeyboardEvent<HTMLInputElement>) => {
      if (event.key === "ArrowDown") {
        event.preventDefault();
        if (visibleItems.length === 0) {
          return;
        }
        setSelectedIndex((previous) => (previous + 1) % visibleItems.length);
        return;
      }

      if (event.key === "ArrowUp") {
        event.preventDefault();
        if (visibleItems.length === 0) {
          return;
        }
        setSelectedIndex((previous) =>
          previous <= 0 ? visibleItems.length - 1 : previous - 1,
        );
        return;
      }

      if (event.key === "Enter") {
        event.preventDefault();
        if (selectedIndex < 0 || selectedIndex >= visibleItems.length) {
          return;
        }
        void executeItem(visibleItems[selectedIndex]);
        return;
      }

      if (event.key === "Escape") {
        event.preventDefault();
        void hideSpotlight();
      }
    },
    [executeItem, hideSpotlight, selectedIndex, visibleItems],
  );

  return (
    <main className="spotlight-shell">
      <section className="spotlight-card">
        <header className="spotlight-header">
          <p className="spotlight-label">Spotlight</p>
          <h1 className="spotlight-title">Search Vanguard</h1>
        </header>

        <label className="spotlight-search" htmlFor="spotlight-search-input">
          <span className="spotlight-search-icon">⌘</span>
          <input
            id="spotlight-search-input"
            className="spotlight-input"
            type="text"
            // biome-ignore lint/a11y/noAutofocus: need autoFocus
            autoFocus
            placeholder="Search passwords, commands, or pages"
            aria-label="Search"
            value={query}
            onChange={(event) => setQuery(event.target.value)}
            onKeyDown={onInputKeyDown}
          />
        </label>

        <ul className="spotlight-results">
          {visibleItems.length === 0 ? (
            <li className="spotlight-empty">No result</li>
          ) : (
            visibleItems.map((item, index) => (
              <li key={item.id}>
                <button
                  type="button"
                  className={`spotlight-item${index === selectedIndex ? " is-active" : ""}`}
                  onClick={() => {
                    void executeItem(item);
                  }}
                  onMouseEnter={() => setSelectedIndex(index)}
                >
                  <div>
                    <p className="spotlight-item-title">{item.title}</p>
                    <p className="spotlight-item-subtitle">{item.subtitle}</p>
                  </div>
                  <span className="spotlight-item-badge">{item.badge}</span>
                </button>
              </li>
            ))
          )}
        </ul>

        <footer className="spotlight-meta">
          <span className="spotlight-meta-item">
            <kbd className="spotlight-kbd">↑</kbd>
            <kbd className="spotlight-kbd">↓</kbd>
            Navigate
          </span>
          <span className="spotlight-meta-item">
            <kbd className="spotlight-kbd">Enter</kbd>
            Run
          </span>
          <span className="spotlight-meta-item">
            <kbd className="spotlight-kbd">Esc</kbd>
            Close
          </span>
          {isLoadingVault && <span className="spotlight-status">Loading vault...</span>}
          {!isLoadingVault && statusText && <span className="spotlight-status">{statusText}</span>}
        </footer>
      </section>
    </main>
  );
}

ReactDOM.createRoot(document.getElementById("spotlight-root") as HTMLElement).render(
  <StrictMode>
    <SpotlightApp />
  </StrictMode>,
);
