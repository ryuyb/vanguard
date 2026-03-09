import type { Dispatch, SetStateAction } from "react";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import {
  commands,
  type VaultCipherDetailDto,
  type VaultViewDataResponseDto,
} from "@/bindings";
import {
  ALL_ITEMS_ID,
  FAVORITES_ID,
  TRASH_ID,
} from "@/features/vault/constants";
import { useCipherDetailSelection } from "@/features/vault/hooks/use-cipher-detail-selection";
import { useExpandedFolderKeys } from "@/features/vault/hooks/use-expanded-folder-keys";
import { useFilteredCiphers } from "@/features/vault/hooks/use-filtered-ciphers";
import { useFolderSelectionGuard } from "@/features/vault/hooks/use-folder-selection-guard";
import { useIconState } from "@/features/vault/hooks/use-icon-state";
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
  getCipherIconUrl,
  sortFolders,
  toAvatarText,
  toVaultErrorText,
} from "@/features/vault/utils";
import { resolveSessionRoute, type SessionRoute } from "@/lib/route-session";

export type VaultPageNavigationTarget = SessionRoute;

type UseVaultPageModelParams = {
  navigateTo: (to: SessionRoute) => Promise<void>;
};

export function useVaultPageModel({ navigateTo }: UseVaultPageModelParams) {
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
  const [iconServer, setIconServer] = useState<string | null>(null);
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

  const {
    setIconLoading,
    setIconLoaded,
    setIconFallback,
    setCipherVisible,
    cleanupStaleStates,
    getIconLoadState,
  } = useIconState();

  const {
    cipherDetailError,
    isCipherDetailLoading,
    loadCipherDetail,
    selectedCipherDetail,
    selectedCipherId,
    useClearSelectionWhenMissing,
  } = useCipherDetailSelection();

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

      const iconServerResult = await commands.vaultGetIconServer();
      if (iconServerResult.status === "ok") {
        setIconServer(iconServerResult.data);
      }

      const result = await commands.vaultGetViewData();

      if (result.status === "error") {
        const text = toVaultErrorText(result.error);
        if (text.toLowerCase().includes("vault is locked")) {
          await navigateTo("/unlock");
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
  }, [navigateTo]);

  useEffect(() => {
    void loadVaultData();
  }, [loadVaultData]);

  const onLock = async () => {
    setIsLocking(true);
    try {
      const result = await commands.vaultLock({});
      if (result.status === "ok") {
        await navigateTo("/unlock");
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

  // Separate icon URL generation from state to optimize re-renders
  const cipherIconUrls = useMemo(
    () =>
      new Map(
        filteredCiphers.map((cipher) => [
          cipher.id,
          getCipherIconUrl(cipher, iconServer),
        ]),
      ),
    [filteredCiphers, iconServer],
  );

  const ciphersWithIcons = useMemo<CipherWithIcon[]>(
    () =>
      filteredCiphers.map((cipher) => {
        const iconUrl = cipherIconUrls.get(cipher.id) ?? null;
        const iconLoadState = getIconLoadState(cipher.id, iconUrl ?? undefined);
        const shouldLoadIcon =
          iconUrl != null &&
          (iconLoadState === "loading" ||
            iconLoadState === "loaded" ||
            iconLoadState === "fallback");

        return {
          ...cipher,
          iconUrl,
          iconLoadState,
          shouldLoadIcon,
        };
      }),
    [cipherIconUrls, filteredCiphers, getIconLoadState],
  );

  const filteredCipherIds = useMemo(
    () => filteredCiphers.map((cipher) => cipher.id),
    [filteredCiphers],
  );

  // Clean up icon state when cipher list changes
  useEffect(() => {
    cleanupStaleStates(filteredCipherIds);
  }, [cleanupStaleStates, filteredCipherIds]);

  const setCipherRowVisible = useCallback(
    (cipherId: string, visible: boolean) => {
      setCipherVisible(cipherId, visible);

      if (visible) {
        const iconUrl = cipherIconUrls.get(cipherId);
        const currentState = getIconLoadState(cipherId, iconUrl ?? undefined);
        if (currentState === "idle") {
          setIconLoading(cipherId);
        }
      }
    },
    [cipherIconUrls, getIconLoadState, setCipherVisible, setIconLoading],
  );

  const markCipherIconLoaded = useCallback(
    (cipherId: string) => {
      const iconUrl = cipherIconUrls.get(cipherId);
      setIconLoaded(cipherId, iconUrl ?? undefined);
    },
    [cipherIconUrls, setIconLoaded],
  );

  const markCipherIconFallback = useCallback(
    (cipherId: string) => {
      const iconUrl = cipherIconUrls.get(cipherId);
      setIconFallback(cipherId, iconUrl ?? undefined);
    },
    [cipherIconUrls, setIconFallback],
  );

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
    iconServer,
    inlineSearchInputRef,
    isCipherDetailLoading,
    isHeaderActionBusy,
    isInlineSearchOpen,
    isLocking,
    isLoggingOut,
    isRefreshing,
    loadCipherDetail,
    lockLabel,
    logoutLabel,
    markCipherIconFallback,
    markCipherIconLoaded,
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
    iconServer: string | null;
    inlineSearchInputRef: typeof inlineSearchInputRef;
    isCipherDetailLoading: boolean;
    isHeaderActionBusy: boolean;
    isInlineSearchOpen: boolean;
    isLocking: boolean;
    isLoggingOut: boolean;
    isRefreshing: boolean;
    loadCipherDetail: (cipherId: string) => Promise<void>;
    lockLabel: string;
    logoutLabel: string;
    markCipherIconFallback: (cipherId: string) => void;
    markCipherIconLoaded: (cipherId: string) => void;
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
