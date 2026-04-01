import { useCallback } from "react";
import { useTranslation } from "react-i18next";
import { commands } from "@/bindings";
import { toast } from "@/lib/toast";
import { useTrashDialogState } from "./use-trash-dialog-state";

type UseTrashOperationsOptions = {
  onSuccess?: () => void;
};

export type TrashOperations = {
  // Dialog state
  isRestoreDialogOpen: boolean;
  isPermanentDeleteDialogOpen: boolean;
  trashActionCipherId: string | null;
  trashActionCipherName: string;
  isTrashActionLoading: boolean;

  // Action handlers
  handleRestoreCipher: (cipherId: string, cipherName: string) => void;
  handlePermanentDeleteCipher: (cipherId: string, cipherName: string) => void;
  handleRestoreConfirm: () => Promise<void>;
  handlePermanentDeleteConfirm: () => Promise<void>;

  // Dialog close handlers (for onOpenChange)
  closeRestoreDialog: () => void;
  closePermanentDeleteDialog: () => void;
};

export function useTrashOperations(
  options?: UseTrashOperationsOptions,
): TrashOperations {
  const { t } = useTranslation();

  const {
    isRestoreDialogOpen,
    isPermanentDeleteDialogOpen,
    trashActionCipherId,
    trashActionCipherName,
    isTrashActionLoading,
    openRestoreDialog,
    openPermanentDeleteDialog,
    closeRestoreDialog,
    closePermanentDeleteDialog,
    setIsTrashActionLoading,
  } = useTrashDialogState();

  const handleRestoreCipher = useCallback(
    (cipherId: string, cipherName: string) => {
      openRestoreDialog(cipherId, cipherName);
    },
    [openRestoreDialog],
  );

  const handlePermanentDeleteCipher = useCallback(
    (cipherId: string, cipherName: string) => {
      openPermanentDeleteDialog(cipherId, cipherName);
    },
    [openPermanentDeleteDialog],
  );

  const handleRestoreConfirm = useCallback(async () => {
    if (!trashActionCipherId) return;

    setIsTrashActionLoading(true);
    const result = await commands.restoreCipher({
      cipherId: trashActionCipherId,
    });
    setIsTrashActionLoading(false);

    if (result.status === "error") {
      toast.error(t("vault.feedback.cipher.restoreError.title"), {
        description: t("vault.feedback.cipher.restoreError.description"),
      });
      return;
    }

    closeRestoreDialog();
    toast.success(t("vault.feedback.cipher.restoreSuccess.title"), {
      description: t("vault.feedback.cipher.restoreSuccess.description", {
        name: trashActionCipherName,
      }),
    });
    options?.onSuccess?.();
  }, [
    trashActionCipherId,
    trashActionCipherName,
    setIsTrashActionLoading,
    closeRestoreDialog,
    t,
    options,
  ]);

  const handlePermanentDeleteConfirm = useCallback(async () => {
    if (!trashActionCipherId) return;

    setIsTrashActionLoading(true);
    const result = await commands.deleteCipher({
      cipherId: trashActionCipherId,
    });
    setIsTrashActionLoading(false);

    if (result.status === "error") {
      toast.error(t("vault.feedback.cipher.permanentDeleteError.title"), {
        description: t(
          "vault.feedback.cipher.permanentDeleteError.description",
        ),
      });
      return;
    }

    closePermanentDeleteDialog();
    toast.success(t("vault.feedback.cipher.permanentDeleteSuccess.title"), {
      description: t(
        "vault.feedback.cipher.permanentDeleteSuccess.description",
        {
          name: trashActionCipherName,
        },
      ),
    });
    options?.onSuccess?.();
  }, [
    trashActionCipherId,
    trashActionCipherName,
    setIsTrashActionLoading,
    closePermanentDeleteDialog,
    t,
    options,
  ]);

  return {
    // Dialog state
    isRestoreDialogOpen,
    isPermanentDeleteDialogOpen,
    trashActionCipherId,
    trashActionCipherName,
    isTrashActionLoading,

    // Action handlers
    handleRestoreCipher,
    handlePermanentDeleteCipher,
    handleRestoreConfirm,
    handlePermanentDeleteConfirm,

    // Dialog close handlers
    closeRestoreDialog,
    closePermanentDeleteDialog,
  };
}
