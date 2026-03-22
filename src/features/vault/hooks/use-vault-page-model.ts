import type { Dispatch, SetStateAction } from "react";
import {
  useCallback,
  useDeferredValue,
  useEffect,
  useMemo,
  useRef,
  useState,
} from "react";
import {
  commands,
  type VaultCipherDetailDto,
  type VaultViewDataResponseDto,
} from "@/bindings";
import { useUnifiedUnlock } from "@/features/auth/unlock/hooks";
import {
  ALL_ITEMS_ID,
  FAVORITES_ID,
  NO_FOLDER_ID,
  TRASH_ID,
} from "@/features/vault/constants";
import { useCipherDetailSelection } from "@/features/vault/hooks/use-cipher-detail-selection";
import { useCipherEvents } from "@/features/vault/hooks/use-cipher-events";
import { useExpandedFolderKeys } from "@/features/vault/hooks/use-expanded-folder-keys";
import { useFilteredCiphers } from "@/features/vault/hooks/use-filtered-ciphers";
import { useFolderSelectionGuard } from "@/features/vault/hooks/use-folder-selection-guard";
import { useFoldersSync } from "@/features/vault/hooks/use-folders-sync";

import { useInlineSearchFocus } from "@/features/vault/hooks/use-inline-search-focus";
import type {
  CipherSortBy,
  CipherSortDirection,
  CipherTypeFilter,
  CipherWithIcon,
  VaultPageState,
} from "@/features/vault/types";
import {
  buildFolderTree,
  collectFolderTreeKeys,
  sortFolders,
  toAvatarText,
  toCipherIconHost,
} from "@/features/vault/utils";
import { appI18n } from "@/i18n";
import { errorHandler } from "@/lib/error-handler";
import { resolveSessionRoute, type SessionRoute } from "@/lib/route-session";

export type VaultPageNavigationTarget = SessionRoute;

type UseVaultPageModelParams = {
  navigateTo: (to: SessionRoute) => Promise<void>;
};

export function useVaultPageModel({ navigateTo }: UseVaultPageModelParams) {
  const searchInputRef = useRef<HTMLInputElement | null>(null);
  const inlineSearchInputRef = useRef<HTMLInputElement | null>(null);
  const [headerSearchQuery, setHeaderSearchQuery] = useState("");
  const deferredHeaderSearchQuery = useDeferredValue(headerSearchQuery);
  const [cipherSearchQuery, setCipherSearchQuery] = useState("");
  const [isInlineSearchOpen, setIsInlineSearchOpen] = useState(false);
  const [typeFilter, setTypeFilter] = useState<CipherTypeFilter>("all");
  const [sortBy, setSortBy] = useState<CipherSortBy>("modified");
  const [sortDirection, setSortDirection] =
    useState<CipherSortDirection>("desc");
  const [isRefreshing, setIsRefreshing] = useState(false);
  const [isLocking, setIsLocking] = useState(false);
  const [isLoggingOut, setIsLoggingOut] = useState(false);
  const [pageState, setPageState] = useState<VaultPageState>("loading");
  const [errorText, setErrorText] = useState("");
  const [viewData, setViewData] = useState<VaultViewDataResponseDto | null>(
    null,
  );
  const [selectedMenuId, setSelectedMenuId] = useState(ALL_ITEMS_ID);
  const [expandedNodeKeys, setExpandedNodeKeys] = useState<Set<string>>(
    new Set<string>(),
  );

  // Unified unlock state
  const {
    state: unlockState,
    isLoading: isUnlockStateLoading,
    isVaultUnlocked,
    isFullyUnlocked,
    isVaultUnlockedSessionExpired,
    lock,
    logout,
    refreshState: refreshUnlockState,
  } = useUnifiedUnlock();

  // Debounced session expired warning to prevent flicker during unlock
  const [showSessionExpiredWarning, setShowSessionExpiredWarning] = useState(false);
  useEffect(() => {
    if (!isVaultUnlockedSessionExpired) {
      setShowSessionExpiredWarning(false);
      return;
    }
    // Delay showing the warning to avoid flicker during unlock
    const timer = setTimeout(() => {
      if (isVaultUnlockedSessionExpired) {
        setShowSessionExpiredWarning(true);
      }
    }, 500);
    return () => clearTimeout(timer);
  }, [isVaultUnlockedSessionExpired]);

  const {
    cipherDetailError,
    isCipherDetailLoading,
    loadCipherDetail,
    selectedCipherDetail,
    selectedCipherId,
    useClearSelectionWhenMissing,
  } = useCipherDetailSelection();

  const selectedCipherIdRef = useRef(selectedCipherId);
  selectedCipherIdRef.current = selectedCipherId;

  // User info from unified state
  const userEmail = useMemo(() => {
    return (
      unlockState?.account?.email ?? appI18n.t("vault.page.user.notSignedIn")
    );
  }, [unlockState?.account?.email]);

  const userBaseUrl = useMemo(() => {
    return (
      unlockState?.account?.baseUrl ??
      appI18n.t("vault.page.user.unknownService")
    );
  }, [unlockState?.account?.baseUrl]);

  const loadCiphersList = useCallback(async () => {
    try {
      const result = await commands.vaultListCiphers();
      if (result.status === "error") {
        errorHandler.handle(result.error);
        return;
      }

      setViewData((prev) => {
        if (!prev) return prev;
        return {
          ...prev,
          ciphers: result.data,
        };
      });
    } catch (error) {
      errorHandler.handle(error);
    }
  }, []);

  const loadVaultData = useCallback(async () => {
    setIsRefreshing(true);
    setErrorText("");
    setPageState("loading");

    try {
      const target = await resolveSessionRoute();
      if (target !== "/vault") {
        await navigateTo(target);
        return;
      }

      // Check unified unlock state and use the returned fresh state
      const freshState = await refreshUnlockState();
      const vaultUnlocked =
        freshState.status === "fullyUnlocked" ||
        freshState.status === "vaultUnlockedSessionExpired";

      if (!vaultUnlocked) {
        // Not unlocked, redirect to unlock page
        await navigateTo("/unlock");
        return;
      }

      // Vault is unlocked, load data
      const result = await commands.vaultGetViewData();

      if (result.status === "error") {
        errorHandler.handle(result.error);
        const errorStr = String(result.error);
        if (errorStr.toLowerCase().includes("vault is locked")) {
          await navigateTo("/unlock");
        } else {
          setPageState("error");
          setErrorText(errorStr);
        }
        setViewData(null);
        return;
      }

      setViewData(result.data);
      setPageState("ready");
    } catch (err) {
      setPageState("error");
      errorHandler.handle(err);
      setViewData(null);
    } finally {
      setIsRefreshing(false);
    }
  }, [navigateTo, refreshUnlockState]);

  useEffect(() => {
    void loadVaultData();
  }, [loadVaultData]);

  // Listen for lock state changes and redirect if locked
  useEffect(() => {
    if (!isUnlockStateLoading && !isVaultUnlocked && pageState === "ready") {
      // State changed to locked while on vault page, redirect to unlock
      void navigateTo("/unlock");
    }
  }, [isUnlockStateLoading, isVaultUnlocked, pageState, navigateTo]);

  // 监听 folders 同步事件，只更新 folders 数据
  useFoldersSync({
    onFoldersSynced: (folders) => {
      // 将 FolderDto[] 转换为 VaultFolderItemDto[]
      const folderItems = folders.map((folder) => ({
        id: folder.id,
        name: folder.name,
      }));

      // 更新 viewData 中的 folders
      setViewData((prev) => {
        if (!prev) return prev;
        return {
          ...prev,
          folders: folderItems,
        };
      });
    },
  });

  // 监听 cipher 事件，实现细粒度更新
  useCipherEvents({
    onCipherCreated: () => {
      // 创建新 cipher 后只重新加载 ciphers 列表
      void loadCiphersList();
    },
    onCipherUpdated: (cipherId) => {
      void loadCiphersList();
      if (selectedCipherIdRef.current === cipherId) {
        void loadCipherDetail(cipherId);
      }
    },
    onCipherDeleted: (cipherId) => {
      // 删除 cipher 时直接从本地状态移除，避免任何网络请求
      setViewData((prev) => {
        if (!prev) return prev;
        return {
          ...prev,
          ciphers: prev.ciphers.filter((cipher) => cipher.id !== cipherId),
        };
      });
    },
  });

  const onLock = async () => {
    setIsLocking(true);
    try {
      const success = await lock();
      if (success) {
        await navigateTo("/unlock");
      }
    } finally {
      setIsLocking(false);
    }
  };

  const onLogout = async () => {
    setIsLoggingOut(true);
    try {
      const success = await logout();
      if (success) {
        await navigateTo("/");
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

  useExpandedFolderKeys({
    folderTree,
    folderTreeKeys,
    setExpandedNodeKeys,
  });

  useFolderSelectionGuard({
    folderIdSet,
    selectedMenuId,
    setSelectedMenuId,
  });

  useInlineSearchFocus({
    inlineSearchInputRef,
    isInlineSearchOpen,
  });

  const filteredCiphers = useFilteredCiphers({
    cipherSearchQuery,
    selectedMenuId,
    sortBy,
    sortDirection,
    typeFilter,
    viewData,
  });

  // Icon data is now loaded directly from backend - no URL construction needed
  const ciphersWithIcons = useMemo<CipherWithIcon[]>(
    () =>
      filteredCiphers.map((cipher) => {
        // Extract hostname from first URI for icon lookup
        const firstUri = cipher.uris?.[0] ?? null;
        const iconHostname = firstUri ? toCipherIconHost(firstUri) : null;
        return {
          ...cipher,
          iconHostname,
        };
      }),
    [filteredCiphers],
  );

  // Header search results - search across all non-deleted ciphers
  const headerSearchResults = useMemo<CipherWithIcon[]>(() => {
    const query = deferredHeaderSearchQuery.trim().toLowerCase();
    if (!query) return [];

    const allCiphers = viewData?.ciphers ?? [];
    const activeCiphers = allCiphers.filter(
      (cipher) => cipher.deletedDate == null,
    );

    const matched = activeCiphers.filter((cipher) => {
      const searchText = [cipher.name, cipher.username, ...(cipher.uris ?? [])]
        .filter(Boolean)
        .join(" ")
        .toLowerCase();
      return searchText.includes(query);
    });

    // Limit to 10 results
    const limited = matched.slice(0, 10);

    return limited.map((cipher) => {
      const firstUri = cipher.uris?.[0] ?? null;
      const iconHostname = firstUri ? toCipherIconHost(firstUri) : null;
      return {
        ...cipher,
        iconHostname,
      };
    });
  }, [deferredHeaderSearchQuery, viewData?.ciphers]);

  // Placeholder callbacks - icon loading is now handled by useIcon hook
  const setCipherRowVisible = useCallback(
    (_cipherId: string, _visible: boolean) => {
      // No-op - visibility tracking not needed with new icon system
    },
    [],
  );

  const markCipherIconLoaded = useCallback((_cipherId: string) => {
    // No-op - loaded state handled by useIcon hook
  }, []);

  const markCipherIconFallback = useCallback((_cipherId: string) => {
    // No-op - error state handled by useIcon hook
  }, []);

  useClearSelectionWhenMissing(filteredCiphers.map((cipher) => cipher.id));

  const folderCipherCount = useMemo(() => {
    const map = new Map<string, number>();
    for (const cipher of viewData?.ciphers ?? []) {
      if (cipher.deletedDate != null || !cipher.folderId) {
        continue;
      }
      map.set(cipher.folderId, (map.get(cipher.folderId) ?? 0) + 1);
    }
    return map;
  }, [viewData?.ciphers]);

  const favoriteCipherCount = useMemo(
    () =>
      (viewData?.ciphers ?? []).filter(
        (cipher) => cipher.favorite === true && cipher.deletedDate == null,
      ).length,
    [viewData?.ciphers],
  );

  const trashCipherCount = useMemo(
    () =>
      (viewData?.ciphers ?? []).filter((cipher) => cipher.deletedDate != null)
        .length,
    [viewData?.ciphers],
  );

  const noFolderCipherCount = useMemo(
    () =>
      (viewData?.ciphers ?? []).filter(
        (cipher) => !cipher.folderId && cipher.deletedDate == null,
      ).length,
    [viewData?.ciphers],
  );

  const selectedMenuName = useMemo(() => {
    if (selectedMenuId === ALL_ITEMS_ID) {
      return appI18n.t("vault.page.menus.allItems");
    }
    if (selectedMenuId === FAVORITES_ID) {
      return appI18n.t("vault.page.menus.favorites");
    }
    if (selectedMenuId === TRASH_ID) {
      return appI18n.t("vault.page.menus.trash");
    }
    if (selectedMenuId === NO_FOLDER_ID) {
      return appI18n.t("vault.page.menus.noFolder");
    }
    return (
      sortedFolders.find((folder) => folder.id === selectedMenuId)?.name ??
      appI18n.t("vault.page.menus.unknownFolder")
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
  const lockLabel = isLocking
    ? appI18n.t("vault.page.actions.locking")
    : appI18n.t("vault.page.actions.lock");
  const logoutLabel = isLoggingOut
    ? appI18n.t("vault.page.actions.loggingOut")
    : appI18n.t("vault.page.actions.logout");

  return {
    avatarText,
    cipherDetailError,
    cipherSearchQuery,
    errorText,
    expandedNodeKeys,
    favoriteCipherCount,
    filteredCiphers: ciphersWithIcons,
    folderCipherCount,
    folderTree,
    headerSearchQuery,
    headerSearchResults,
    inlineSearchInputRef,
    isCipherDetailLoading,
    isFullyUnlocked,
    isHeaderActionBusy,
    isInlineSearchOpen,
    isLocking,
    isLoggingOut,
    isRefreshing,
    isVaultUnlockedSessionExpired: showSessionExpiredWarning,
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
  } as {
    avatarText: string;
    cipherDetailError: string;
    cipherSearchQuery: string;
    errorText: string;
    expandedNodeKeys: Set<string>;
    favoriteCipherCount: number;
    filteredCiphers: typeof ciphersWithIcons;
    folderCipherCount: Map<string, number>;
    folderTree: ReturnType<typeof buildFolderTree>;
    headerSearchQuery: string;
    headerSearchResults: CipherWithIcon[];
    inlineSearchInputRef: typeof inlineSearchInputRef;
    isCipherDetailLoading: boolean;
    isFullyUnlocked: boolean;
    isHeaderActionBusy: boolean;
    isInlineSearchOpen: boolean;
    isLocking: boolean;
    isLoggingOut: boolean;
    isRefreshing: boolean;
    isVaultUnlockedSessionExpired: boolean;
    loadCipherDetail: (cipherId: string) => Promise<void>;
    loadVaultData: () => Promise<void>;
    lockLabel: string;
    logoutLabel: string;
    markCipherIconFallback: (cipherId: string) => void;
    markCipherIconLoaded: (cipherId: string) => void;
    noFolderCipherCount: number;
    onFolderTreeOpenChange: (nodeKey: string, open: boolean) => void;
    onLock: () => Promise<void>;
    onLogout: () => Promise<void>;
    pageState: VaultPageState;
    searchInputRef: typeof searchInputRef;
    selectedCipherDetail: VaultCipherDetailDto | null;
    selectedCipherId: string | null;
    selectedMenuId: string;
    selectedMenuName: string;
    setCipherRowVisible: (cipherId: string, visible: boolean) => void;
    setCipherSearchQuery: Dispatch<SetStateAction<string>>;
    setHeaderSearchQuery: Dispatch<SetStateAction<string>>;
    setIsInlineSearchOpen: Dispatch<SetStateAction<boolean>>;
    setSelectedMenuId: Dispatch<SetStateAction<string>>;
    setSortBy: Dispatch<SetStateAction<CipherSortBy>>;
    setSortDirection: Dispatch<SetStateAction<CipherSortDirection>>;
    setTypeFilter: Dispatch<SetStateAction<CipherTypeFilter>>;
    sortBy: CipherSortBy;
    sortDirection: CipherSortDirection;
    trashCipherCount: number;
    typeFilter: CipherTypeFilter;
    userBaseUrl: string;
    userEmail: string;
    viewData: VaultViewDataResponseDto | null;
  };
}
