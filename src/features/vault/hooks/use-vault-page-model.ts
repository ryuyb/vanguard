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
import type {
  CipherSortBy,
  CipherSortDirection,
  CipherTypeFilter,
  VaultPageState,
} from "@/features/vault/types";
import {
  buildFolderTree,
  collectFolderTreeKeys,
  sortFolders,
  toAvatarText,
  toSortableDate,
  toVaultErrorText,
} from "@/features/vault/utils";
import { resolveSessionRoute, type SessionRoute } from "@/lib/route-session";

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
  const [isRefreshing, setIsRefreshing] = useState(false);
  const [isLocking, setIsLocking] = useState(false);
  const [isLoggingOut, setIsLoggingOut] = useState(false);
  const [pageState, setPageState] = useState<VaultPageState>("loading");
  const [errorText, setErrorText] = useState("");
  const [viewData, setViewData] = useState<VaultViewDataResponseDto | null>(
    null,
  );
  const [selectedMenuId, setSelectedMenuId] = useState(ALL_ITEMS_ID);
  const [selectedCipherId, setSelectedCipherId] = useState<string | null>(null);
  const [selectedCipherDetail, setSelectedCipherDetail] =
    useState<VaultCipherDetailDto | null>(null);
  const [isCipherDetailLoading, setIsCipherDetailLoading] = useState(false);
  const [cipherDetailError, setCipherDetailError] = useState("");
  const [expandedNodeKeys, setExpandedNodeKeys] = useState<Set<string>>(
    new Set<string>(),
  );
  const detailRequestSeqRef = useRef(0);

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

  useEffect(() => {
    setExpandedNodeKeys((previous) => {
      const validKeys = new Set(folderTreeKeys);
      const next = new Set<string>();
      for (const key of previous) {
        if (validKeys.has(key)) {
          next.add(key);
        }
      }
      if (next.size === 0) {
        for (const node of folderTree) {
          next.add(node.key);
        }
      }
      return next;
    });
  }, [folderTree, folderTreeKeys]);

  useEffect(() => {
    if (
      selectedMenuId === ALL_ITEMS_ID ||
      selectedMenuId === FAVORITES_ID ||
      selectedMenuId === TRASH_ID
    ) {
      return;
    }
    const exists = folderIdSet.has(selectedMenuId);
    if (!exists) {
      setSelectedMenuId(ALL_ITEMS_ID);
    }
  }, [folderIdSet, selectedMenuId]);

  useEffect(() => {
    if (!isInlineSearchOpen) {
      return;
    }
    const frameId = requestAnimationFrame(() => {
      inlineSearchInputRef.current?.focus();
    });
    return () => {
      cancelAnimationFrame(frameId);
    };
  }, [isInlineSearchOpen]);

  const normalizedCipherSearchQuery = cipherSearchQuery.trim().toLowerCase();

  const filteredCiphers = useMemo(() => {
    const allCiphers = viewData?.ciphers ?? [];
    const folderFiltered =
      selectedMenuId === ALL_ITEMS_ID ||
      selectedMenuId === FAVORITES_ID ||
      selectedMenuId === TRASH_ID
        ? allCiphers
        : allCiphers.filter((cipher) => cipher.folderId === selectedMenuId);

    const typeFiltered = folderFiltered.filter((cipher) => {
      if (typeFilter === "all") {
        return true;
      }
      if (typeFilter === "login") {
        return cipher.type === 1;
      }
      if (typeFilter === "note") {
        return cipher.type === 2;
      }
      if (typeFilter === "card") {
        return cipher.type === 3;
      }
      if (typeFilter === "identify") {
        return cipher.type === 4;
      }
      if (typeFilter === "ssh_key") {
        return cipher.type === 5;
      }
      return true;
    });

    const searchFiltered = !normalizedCipherSearchQuery
      ? typeFiltered
      : typeFiltered.filter((cipher) => {
          const searchText = [cipher.name, cipher.id, cipher.organizationId]
            .filter(Boolean)
            .join(" ")
            .toLowerCase();
          return searchText.includes(normalizedCipherSearchQuery);
        });

    return [...searchFiltered].sort((left, right) => {
      if (sortBy === "title") {
        const titleCompare = (left.name ?? "").localeCompare(
          right.name ?? "",
          "zh-Hans-CN",
        );
        return sortDirection === "asc" ? titleCompare : -titleCompare;
      }
      if (sortBy === "created") {
        const createdCompare =
          toSortableDate(left.creationDate) -
          toSortableDate(right.creationDate);
        return sortDirection === "asc" ? createdCompare : -createdCompare;
      }
      const modifiedCompare =
        toSortableDate(left.revisionDate) - toSortableDate(right.revisionDate);
      return sortDirection === "asc" ? modifiedCompare : -modifiedCompare;
    });
  }, [
    normalizedCipherSearchQuery,
    selectedMenuId,
    sortBy,
    sortDirection,
    typeFilter,
    viewData?.ciphers,
  ]);

  useEffect(() => {
    if (!selectedCipherId) {
      return;
    }
    const existsInList = filteredCiphers.some(
      (cipher) => cipher.id === selectedCipherId,
    );
    if (existsInList) {
      return;
    }
    detailRequestSeqRef.current += 1;
    setSelectedCipherId(null);
    setSelectedCipherDetail(null);
    setCipherDetailError("");
    setIsCipherDetailLoading(false);
  }, [filteredCiphers, selectedCipherId]);

  const loadCipherDetail = useCallback(async (cipherId: string) => {
    const normalizedCipherId = cipherId.trim();
    if (!normalizedCipherId) {
      return;
    }

    setSelectedCipherId(normalizedCipherId);
    setSelectedCipherDetail(null);
    setCipherDetailError("");
    setIsCipherDetailLoading(true);

    const requestSeq = detailRequestSeqRef.current + 1;
    detailRequestSeqRef.current = requestSeq;

    try {
      const detail = await commands.vaultGetCipherDetail({
        cipherId: normalizedCipherId,
      });

      if (requestSeq !== detailRequestSeqRef.current) {
        return;
      }

      if (detail.status === "error") {
        setCipherDetailError(toVaultErrorText(detail.error));
        return;
      }

      setSelectedCipherDetail(detail.data.cipher);
    } catch (error) {
      if (requestSeq !== detailRequestSeqRef.current) {
        return;
      }
      setCipherDetailError(toVaultErrorText(error));
    } finally {
      if (requestSeq === detailRequestSeqRef.current) {
        setIsCipherDetailLoading(false);
      }
    }
  }, []);

  const folderCipherCount = useMemo(() => {
    const map = new Map<string, number>();
    for (const cipher of viewData?.ciphers ?? []) {
      if (!cipher.folderId) {
        continue;
      }
      map.set(cipher.folderId, (map.get(cipher.folderId) ?? 0) + 1);
    }
    return map;
  }, [viewData?.ciphers]);

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
    isRefreshing,
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
  } as {
    avatarText: string;
    cipherDetailError: string;
    cipherSearchQuery: string;
    errorText: string;
    expandedNodeKeys: Set<string>;
    filteredCiphers: NonNullable<VaultViewDataResponseDto["ciphers"]>;
    folderCipherCount: Map<string, number>;
    folderTree: ReturnType<typeof buildFolderTree>;
    headerSearchQuery: string;
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
    onFolderTreeOpenChange: (nodeKey: string, open: boolean) => void;
    onLock: () => Promise<void>;
    onLogout: () => Promise<void>;
    pageState: VaultPageState;
    searchInputRef: typeof searchInputRef;
    selectedCipherDetail: VaultCipherDetailDto | null;
    selectedCipherId: string | null;
    selectedMenuId: string;
    selectedMenuName: string;
    setCipherSearchQuery: Dispatch<SetStateAction<string>>;
    setHeaderSearchQuery: Dispatch<SetStateAction<string>>;
    setIsInlineSearchOpen: Dispatch<SetStateAction<boolean>>;
    setSelectedMenuId: Dispatch<SetStateAction<string>>;
    setSortBy: Dispatch<SetStateAction<CipherSortBy>>;
    setSortDirection: Dispatch<SetStateAction<CipherSortDirection>>;
    setTypeFilter: Dispatch<SetStateAction<CipherTypeFilter>>;
    sortBy: CipherSortBy;
    sortDirection: CipherSortDirection;
    typeFilter: CipherTypeFilter;
    userBaseUrl: string;
    userEmail: string;
    viewData: VaultViewDataResponseDto | null;
  };
}
