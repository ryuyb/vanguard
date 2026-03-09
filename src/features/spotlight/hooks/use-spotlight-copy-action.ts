import { useCallback, useState } from "react";
import { commands } from "@/bindings";
import { COPY_FLASH_DURATION_MS } from "@/features/spotlight/constants";
import type { CopyField, SpotlightItem } from "@/features/spotlight/types";
import { errorHandler } from "@/lib/error-handler";

type UseSpotlightCopyActionParams = {
  hideSpotlight: () => Promise<void>;
};

type UseSpotlightCopyActionResult = {
  copiedDetailField: CopyField | null;
  copiedItemId: string | null;
  isCopying: boolean;
  runCopyAction: (item: SpotlightItem, field: CopyField) => Promise<void>;
};

export function useSpotlightCopyAction({
  hideSpotlight,
}: UseSpotlightCopyActionParams): UseSpotlightCopyActionResult {
  const [copiedItemId, setCopiedItemId] = useState<string | null>(null);
  const [copiedDetailField, setCopiedDetailField] = useState<CopyField | null>(
    null,
  );
  const [isCopying, setIsCopying] = useState(false);

  const runCopyAction = useCallback(
    async (item: SpotlightItem, field: CopyField) => {
      if (isCopying) {
        return;
      }

      setIsCopying(true);
      try {
        const result = await commands.vaultCopyCipherField({
          cipherId: item.cipherId,
          field,
          clearAfterMs: null,
        });
        if (result.status === "error") {
          errorHandler.handle(result.error);
          return;
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
        errorHandler.handle(error);
      } finally {
        setIsCopying(false);
      }
    },
    [hideSpotlight, isCopying],
  );

  return {
    copiedDetailField,
    copiedItemId,
    isCopying,
    runCopyAction,
  };
}
