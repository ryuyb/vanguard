import { useMemo } from "react";
import type { VaultViewDataResponseDto } from "@/bindings";
import {
  ALL_ITEMS_ID,
  FAVORITES_ID,
  TRASH_ID,
} from "@/features/vault/constants";
import type {
  CipherSortBy,
  CipherSortDirection,
  CipherTypeFilter,
} from "@/features/vault/types";
import { toSortableDate } from "@/features/vault/utils";

export function useFilteredCiphers({
  cipherSearchQuery,
  selectedMenuId,
  sortBy,
  sortDirection,
  typeFilter,
  viewData,
}: {
  cipherSearchQuery: string;
  selectedMenuId: string;
  sortBy: CipherSortBy;
  sortDirection: CipherSortDirection;
  typeFilter: CipherTypeFilter;
  viewData: VaultViewDataResponseDto | null;
}) {
  const normalizedCipherSearchQuery = cipherSearchQuery.trim().toLowerCase();

  return useMemo(() => {
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
}
