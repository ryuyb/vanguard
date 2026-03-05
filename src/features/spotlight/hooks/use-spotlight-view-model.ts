import type { Dispatch, SetStateAction } from "react";
import { useEffect, useMemo, useState } from "react";
import type { SpotlightItem } from "@/features/spotlight/types";

type UseSpotlightViewModelParams = {
  isLoadingVault: boolean;
  vaultItems: SpotlightItem[];
};

type UseSpotlightViewModelResult = {
  detailItem: SpotlightItem | null;
  hasQuery: boolean;
  hasVisibleResults: boolean;
  query: string;
  setDetailItemId: Dispatch<SetStateAction<string | null>>;
  setQuery: Dispatch<SetStateAction<string>>;
  shouldShowResults: boolean;
  visibleItems: SpotlightItem[];
};

export function useSpotlightViewModel({
  isLoadingVault,
  vaultItems,
}: UseSpotlightViewModelParams): UseSpotlightViewModelResult {
  const [query, setQuery] = useState("");
  const [detailItemId, setDetailItemId] = useState<string | null>(null);

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
  const shouldShowResults = (isLoadingVault && hasQuery) || hasVisibleResults;

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

  return {
    detailItem,
    hasQuery,
    hasVisibleResults,
    query,
    setDetailItemId,
    setQuery,
    shouldShowResults,
    visibleItems,
  };
}
