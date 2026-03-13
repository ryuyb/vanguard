import { useMemo } from "react";
import type { VaultCipherItemDto, VaultViewDataResponseDto } from "@/bindings";
import {
  ALL_ITEMS_ID,
  FAVORITES_ID,
  NO_FOLDER_ID,
  TRASH_ID,
} from "@/features/vault/constants";
import type {
  CipherSortBy,
  CipherSortDirection,
  CipherTypeFilter,
} from "@/features/vault/types";
import { toSortableDate } from "@/features/vault/utils";
import { getLocaleCollator } from "@/i18n";

function isDeleted(cipher: VaultCipherItemDto) {
  return cipher.deletedDate != null;
}

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
    const menuFiltered = allCiphers.filter((cipher) => {
      if (selectedMenuId === ALL_ITEMS_ID) {
        return !isDeleted(cipher);
      }
      if (selectedMenuId === FAVORITES_ID) {
        return cipher.favorite === true && !isDeleted(cipher);
      }
      if (selectedMenuId === TRASH_ID) {
        return isDeleted(cipher);
      }
      if (selectedMenuId === NO_FOLDER_ID) {
        return !cipher.folderId && !isDeleted(cipher);
      }
      return cipher.folderId === selectedMenuId && !isDeleted(cipher);
    });

    const typeFiltered = menuFiltered.filter((cipher) => {
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
        const collator = getLocaleCollator();
        const titleCompare = collator.compare(
          left.name ?? "",
          right.name ?? "",
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
