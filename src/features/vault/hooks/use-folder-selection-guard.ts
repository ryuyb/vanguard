import type { Dispatch, SetStateAction } from "react";
import { useEffect } from "react";
import {
  ALL_ITEMS_ID,
  FAVORITES_ID,
  TRASH_ID,
} from "@/features/vault/constants";

export function useFolderSelectionGuard({
  folderIdSet,
  selectedMenuId,
  setSelectedMenuId,
}: {
  folderIdSet: Set<string>;
  selectedMenuId: string;
  setSelectedMenuId: Dispatch<SetStateAction<string>>;
}) {
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
  }, [folderIdSet, selectedMenuId, setSelectedMenuId]);
}
