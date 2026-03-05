import type { Dispatch, SetStateAction } from "react";
import { useEffect, useMemo, useState } from "react";
import { commands } from "@/bindings";
import { DETAIL_ACTIONS } from "@/features/spotlight/constants";
import { logClientError } from "@/features/spotlight/logging";
import type { DetailAction, SpotlightItem } from "@/features/spotlight/types";

type UseSpotlightDetailActionsParams = {
  detailItem: SpotlightItem | null;
};

type UseSpotlightDetailActionsResult = {
  detailActionIndex: number;
  detailActions: readonly DetailAction[];
  detailHasTotp: boolean;
  setDetailActionIndex: Dispatch<SetStateAction<number>>;
};

export function useSpotlightDetailActions({
  detailItem,
}: UseSpotlightDetailActionsParams): UseSpotlightDetailActionsResult {
  const [detailActionIndex, setDetailActionIndex] = useState(0);
  const [detailHasTotp, setDetailHasTotp] = useState(false);

  useEffect(() => {
    if (!detailItem) {
      setDetailActionIndex(0);
    }
  }, [detailItem]);

  useEffect(() => {
    let disposed = false;

    setDetailHasTotp(false);
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
          setDetailHasTotp(result.data.cipher.hasTotp);
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
    if (!detailHasTotp) {
      return DETAIL_ACTIONS.filter((action) => !action.requiresTotp);
    }
    return DETAIL_ACTIONS;
  }, [detailHasTotp]);

  useEffect(() => {
    setDetailActionIndex((current) => {
      const maxIndex = Math.max(0, detailActions.length - 1);
      return Math.min(current, maxIndex);
    });
  }, [detailActions.length]);

  return {
    detailActionIndex,
    detailActions,
    detailHasTotp,
    setDetailActionIndex,
  };
}
