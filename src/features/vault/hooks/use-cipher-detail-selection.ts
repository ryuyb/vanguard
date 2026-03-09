import { useCallback, useEffect, useRef, useState } from "react";
import { commands, type VaultCipherDetailDto } from "@/bindings";
import { errorHandler } from "@/lib/error-handler";

export function useCipherDetailSelection() {
  const [selectedCipherId, setSelectedCipherId] = useState<string | null>(null);
  const [selectedCipherDetail, setSelectedCipherDetail] =
    useState<VaultCipherDetailDto | null>(null);
  const [isCipherDetailLoading, setIsCipherDetailLoading] = useState(false);
  const [cipherDetailError, setCipherDetailError] = useState("");
  const detailRequestSeqRef = useRef(0);

  const clearCipherSelection = useCallback(() => {
    detailRequestSeqRef.current += 1;
    setSelectedCipherId(null);
    setSelectedCipherDetail(null);
    setCipherDetailError("");
    setIsCipherDetailLoading(false);
  }, []);

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
        errorHandler.handle(detail.error);
        return;
      }

      setSelectedCipherDetail(detail.data.cipher);
    } catch (error) {
      if (requestSeq !== detailRequestSeqRef.current) {
        return;
      }
      errorHandler.handle(error);
    } finally {
      if (requestSeq === detailRequestSeqRef.current) {
        setIsCipherDetailLoading(false);
      }
    }
  }, []);

  const useClearSelectionWhenMissing = (filteredCipherIds: string[]) => {
    useEffect(() => {
      if (!selectedCipherId) {
        return;
      }
      const existsInList = filteredCipherIds.includes(selectedCipherId);
      if (existsInList) {
        return;
      }
      clearCipherSelection();
    }, [filteredCipherIds]);
  };

  return {
    cipherDetailError,
    clearCipherSelection,
    isCipherDetailLoading,
    loadCipherDetail,
    selectedCipherDetail,
    selectedCipherId,
    setSelectedCipherId,
    useClearSelectionWhenMissing,
  };
}
