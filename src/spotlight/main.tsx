import { getCurrentWindow } from "@tauri-apps/api/window";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import { error as logError } from "@tauri-apps/plugin-log";
import { Command as CommandPrimitive } from "cmdk";
import { SearchIcon } from "lucide-react";
import type { KeyboardEvent } from "react";
import { StrictMode, useCallback, useEffect, useMemo, useState } from "react";
import ReactDOM from "react-dom/client";
import {
  commands,
  type VaultCipherDetailDto,
  type VaultCipherItemDto,
} from "@/bindings";
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
  cipherId: string;
  title: string;
  subtitle: string;
  searchText: string;
};

type CopyField = "username" | "password" | "totp";

type DetailAction = {
  label: string;
  shortcut: readonly string[];
  field: CopyField;
  requiresTotp?: boolean;
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

const DETAIL_ACTIONS: readonly DetailAction[] = [
  { label: "复制 用户名", shortcut: ["⌘", "C"], field: "username" },
  { label: "复制 密码", shortcut: ["⌘", "⇧", "C"], field: "password" },
  {
    label: "复制 一次性密码",
    shortcut: ["⌘", "⌥", "C"],
    field: "totp",
    requiresTotp: true,
  },
];
const COPY_FLASH_DURATION_MS = 180;
const BASE32_ALPHABET = "ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";

type TotpHashAlgorithm = "SHA-1" | "SHA-256" | "SHA-512";

type TotpConfig = {
  secret: Uint8Array;
  hashAlgorithm: TotpHashAlgorithm;
  digits: number;
  period: number;
};

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

function toCipherTotpRaw(cipher: VaultCipherDetailDto) {
  return firstNonEmptyText(cipher.login?.totp, cipher.data?.totp);
}

function parsePositiveInteger(value: string) {
  if (!/^\d+$/.test(value)) {
    return null;
  }
  const parsed = Number.parseInt(value, 10);
  if (!Number.isSafeInteger(parsed)) {
    return null;
  }
  return parsed;
}

function normalizeTotpHashAlgorithm(value: string | null) {
  if (!value) {
    return "SHA-1" as TotpHashAlgorithm;
  }
  const normalized = value.replace(/[^a-zA-Z0-9]/g, "").toUpperCase();
  if (normalized === "SHA1") {
    return "SHA-1" as TotpHashAlgorithm;
  }
  if (normalized === "SHA256") {
    return "SHA-256" as TotpHashAlgorithm;
  }
  if (normalized === "SHA512") {
    return "SHA-512" as TotpHashAlgorithm;
  }
  return null;
}

function decodeBase32Secret(value: string) {
  const sanitized = value
    .trim()
    .toUpperCase()
    .replace(/[\s-]+/g, "")
    .replace(/=+$/g, "");
  if (!sanitized) {
    return null;
  }

  const output: number[] = [];
  let buffer = 0;
  let bitsInBuffer = 0;

  for (const character of sanitized) {
    const index = BASE32_ALPHABET.indexOf(character);
    if (index === -1) {
      return null;
    }

    buffer = (buffer << 5) | index;
    bitsInBuffer += 5;

    while (bitsInBuffer >= 8) {
      output.push((buffer >>> (bitsInBuffer - 8)) & 0xff);
      bitsInBuffer -= 8;
    }
  }

  if (output.length === 0) {
    return null;
  }

  return new Uint8Array(output);
}

function parseTotpConfig(rawTotp: string | null) {
  const raw = (rawTotp ?? "").trim();
  if (!raw) {
    return null;
  }

  let secretText: string | null = null;
  let hashAlgorithm: TotpHashAlgorithm = "SHA-1";
  let digits = 6;
  let period = 30;

  if (raw.toLowerCase().startsWith("otpauth://")) {
    let url: URL;
    try {
      url = new URL(raw);
    } catch {
      return null;
    }

    if (url.protocol !== "otpauth:" || url.hostname.toLowerCase() !== "totp") {
      return null;
    }

    secretText = url.searchParams.get("secret");
    if (!secretText) {
      return null;
    }

    const parsedAlgorithm = normalizeTotpHashAlgorithm(
      url.searchParams.get("algorithm"),
    );
    if (!parsedAlgorithm) {
      return null;
    }
    hashAlgorithm = parsedAlgorithm;

    const digitsParam = url.searchParams.get("digits");
    if (digitsParam) {
      const parsedDigits = parsePositiveInteger(digitsParam);
      if (parsedDigits == null || parsedDigits < 6 || parsedDigits > 10) {
        return null;
      }
      digits = parsedDigits;
    }

    const periodParam = url.searchParams.get("period");
    if (periodParam) {
      const parsedPeriod = parsePositiveInteger(periodParam);
      if (parsedPeriod == null || parsedPeriod <= 0 || parsedPeriod > 300) {
        return null;
      }
      period = parsedPeriod;
    }
  } else {
    secretText = raw;
  }

  const secret = decodeBase32Secret(secretText);
  if (!secret) {
    return null;
  }

  return {
    secret,
    hashAlgorithm,
    digits,
    period,
  } satisfies TotpConfig;
}

async function createCurrentTotpCode(rawTotp: string | null) {
  const config = parseTotpConfig(rawTotp);
  if (!config) {
    return null;
  }

  const key = await crypto.subtle.importKey(
    "raw",
    config.secret,
    {
      name: "HMAC",
      hash: { name: config.hashAlgorithm },
    },
    false,
    ["sign"],
  );

  const unixSeconds = Math.floor(Date.now() / 1000);
  const counter = Math.floor(unixSeconds / config.period);
  const data = new ArrayBuffer(8);
  const view = new DataView(data);
  const high = Math.floor(counter / 0x1_0000_0000);
  const low = counter >>> 0;
  view.setUint32(0, high);
  view.setUint32(4, low);

  const signature = new Uint8Array(await crypto.subtle.sign("HMAC", key, data));
  const offset = signature[signature.length - 1] & 0x0f;
  const binary =
    ((signature[offset] & 0x7f) << 24) |
    ((signature[offset + 1] & 0xff) << 16) |
    ((signature[offset + 2] & 0xff) << 8) |
    (signature[offset + 3] & 0xff);

  const divisor = 10 ** config.digits;
  const otp = binary % divisor;
  return otp.toString().padStart(config.digits, "0");
}

function toCipherItem(cipher: VaultCipherItemDto): SpotlightItem {
  const rawName = cipher.name?.trim() ?? "";
  const rawUsername = cipher.username?.trim() ?? "";
  const title = rawName || "Untitled Cipher";
  const subtitle = rawUsername || "Vault item";
  return {
    id: `cipher-${cipher.id}`,
    cipherId: cipher.id,
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
  const [copiedItemId, setCopiedItemId] = useState<string | null>(null);
  const [copiedDetailField, setCopiedDetailField] = useState<CopyField | null>(
    null,
  );
  const [detailTotpRaw, setDetailTotpRaw] = useState<string | null>(null);
  const [isCopying, setIsCopying] = useState(false);

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

  useEffect(() => {
    let disposed = false;

    setDetailTotpRaw(null);
    if (!detailItem) {
      return () => {
        disposed = true;
      };
    }

    const loadDetailTotp = async () => {
      try {
        const result = await commands.vaultGetCipherDetail({
          cipherId: detailItem.cipherId,
        });
        if (result.status === "error") {
          logClientError("Failed to load cipher detail for totp", result.error);
          return;
        }

        if (!disposed) {
          setDetailTotpRaw(toCipherTotpRaw(result.data.cipher));
        }
      } catch (error) {
        logClientError("Failed to load cipher detail for totp", error);
      }
    };

    void loadDetailTotp();

    return () => {
      disposed = true;
    };
  }, [detailItem]);

  const detailActions = useMemo(() => {
    if (!detailTotpRaw) {
      return DETAIL_ACTIONS.filter((action) => !action.requiresTotp);
    }
    return DETAIL_ACTIONS;
  }, [detailTotpRaw]);

  useEffect(() => {
    setDetailActionIndex((current) => {
      const maxIndex = Math.max(0, detailActions.length - 1);
      return Math.min(current, maxIndex);
    });
  }, [detailActions.length]);

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

  const runCopyAction = useCallback(
    async (item: SpotlightItem, field: CopyField) => {
      if (isCopying) {
        return;
      }

      setIsCopying(true);
      try {
        if (field === "totp") {
          const cachedRawTotp =
            detailItem?.cipherId === item.cipherId ? detailTotpRaw : null;
          const rawTotp =
            cachedRawTotp ??
            (await (async () => {
              const result = await commands.vaultGetCipherDetail({
                cipherId: item.cipherId,
              });
              if (result.status === "error") {
                logClientError(
                  "Failed to load cipher detail when copying totp",
                  result.error,
                );
                return null;
              }
              return toCipherTotpRaw(result.data.cipher);
            })());

          const totpCode = await createCurrentTotpCode(rawTotp);
          if (!totpCode) {
            return;
          }

          await writeText(totpCode);
        } else {
          const result = await commands.vaultCopyCipherField({
            cipherId: item.cipherId,
            field,
            clearAfterMs: null,
          });
          if (result.status === "error") {
            logClientError("Failed to copy cipher field", result.error);
            return;
          }
        }

        setCopiedItemId(item.id);
        setCopiedDetailField(field);
        await new Promise((resolve) =>
          window.setTimeout(resolve, COPY_FLASH_DURATION_MS),
        );
        setCopiedItemId(null);
        setCopiedDetailField(null);
        await hideSpotlight();
      } catch (error) {
        logClientError("Failed to copy cipher field", error);
      } finally {
        setIsCopying(false);
      }
    },
    [detailItem?.cipherId, detailTotpRaw, hideSpotlight, isCopying],
  );

  const onCommandInputKeyDown = useCallback(
    (event: KeyboardEvent<HTMLInputElement>) => {
      const normalizedKey = event.key.toLowerCase();
      const isCopyShortcut =
        (event.metaKey || event.ctrlKey) && normalizedKey === "c";
      if (isCopyShortcut) {
        let field: CopyField | null = null;
        if (event.altKey && !event.shiftKey) {
          field = "totp";
        } else if (event.shiftKey && !event.altKey) {
          field = "password";
        } else if (!event.shiftKey && !event.altKey) {
          field = "username";
        }
        if (!field) {
          return;
        }

        if (field === "totp" && detailItem && !detailTotpRaw) {
          return;
        }

        event.preventDefault();
        if (detailItem) {
          void runCopyAction(detailItem, field);
          return;
        }
        const selectedItemId = resolveSelectedItemId();
        if (!selectedItemId) {
          return;
        }
        const selectedItem =
          visibleItems.find((item) => item.id === selectedItemId) ?? null;
        if (!selectedItem) {
          return;
        }
        void runCopyAction(selectedItem, field);
        return;
      }

      if (event.key === "Enter" && detailItem) {
        event.preventDefault();
        const field = detailActions[detailActionIndex]?.field ?? "username";
        void runCopyAction(detailItem, field);
        return;
      }

      if (
        detailItem &&
        (event.key === "ArrowDown" || event.key === "ArrowUp")
      ) {
        event.preventDefault();
        setDetailActionIndex((current) => {
          if (event.key === "ArrowDown") {
            return (current + 1) % detailActions.length;
          }
          return (current - 1 + detailActions.length) % detailActions.length;
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
      detailActions,
      detailActionIndex,
      detailItem,
      detailTotpRaw,
      hasVisibleResults,
      hideSpotlight,
      query,
      resolveSelectedItemId,
      runCopyAction,
      visibleItems,
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
                    {detailActions.map((action, index) => (
                      <div
                        key={action.label}
                        role="option"
                        tabIndex={-1}
                        aria-selected={detailActionIndex === index}
                        className={`spotlight-detail-action${detailActionIndex === index ? " is-selected" : ""}${copiedDetailField === action.field ? " is-copied" : ""}`}
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
                      className={`spotlight-item${copiedItemId === item.id ? " is-copied" : ""}`}
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
