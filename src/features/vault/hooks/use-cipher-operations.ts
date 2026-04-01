import { useCallback } from "react";
import { useTranslation } from "react-i18next";
import type { SyncCipher, VaultCipherDetailDto } from "@/bindings";
import { useCipherDialogState } from "@/features/vault/hooks/use-cipher-dialog-state";
import { useCipherMutations } from "@/features/vault/hooks/use-cipher-mutations";
import { toast } from "@/lib/toast";

export type CipherOperations = {
  // Dialog state
  cipherFormMode: "create" | "edit";
  isCipherFormOpen: boolean;
  isDeleteCipherDialogOpen: boolean;
  selectedCipherForEdit: SyncCipher | null;
  selectedCipherIdForDelete: string | null;
  selectedCipherNameForDelete: string;

  // Mutation state
  isCreating: boolean;
  isUpdating: boolean;
  isDeleting: boolean;

  // Operation handlers
  handleCreateCipher: () => void;
  handleEditCipher: (cipher: VaultCipherDetailDto) => void;
  handleDeleteCipher: (cipherId: string, cipherName: string) => void;
  handleCloneCipher: (cipher: VaultCipherDetailDto) => Promise<void>;
  handleCipherFormConfirm: (cipher: SyncCipher) => Promise<void>;
  handleDeleteCipherConfirm: () => Promise<void>;

  // Dialog close handlers (for onOpenChange)
  closeCipherFormDialog: () => void;
  closeDeleteCipherDialog: () => void;
};

export function useCipherOperations(): CipherOperations {
  const { t } = useTranslation();

  const dialogState = useCipherDialogState();
  const cipherMutations = useCipherMutations();

  const handleCreateCipher = useCallback(() => {
    dialogState.openCreateCipherDialog();
  }, [dialogState]);

  const handleEditCipher = useCallback(
    (cipher: VaultCipherDetailDto) => {
      dialogState.openEditCipherDialog(cipher);
    },
    [dialogState],
  );

  const handleDeleteCipher = useCallback(
    (cipherId: string, cipherName: string) => {
      dialogState.openDeleteCipherDialog(cipherId, cipherName);
    },
    [dialogState],
  );

  const handleCloneCipher = useCallback(
    async (cipher: VaultCipherDetailDto) => {
      const cloneSuffix = t("vault.page.cipher.cloneSuffix");
      await dialogState.cloneCipher(cipher.id, cloneSuffix);
    },
    [dialogState, t],
  );

  const handleCipherFormConfirm = useCallback(
    async (cipher: SyncCipher) => {
      const cipherName = cipher.name ?? t("vault.page.cipher.untitled");

      try {
        if (dialogState.cipherFormMode === "create") {
          await cipherMutations.createCipher.mutateAsync(cipher);
          toast.success(t("vault.feedback.cipher.createSuccess.title"), {
            description: t("vault.feedback.cipher.createSuccess.description", {
              name: cipherName,
            }),
          });
        } else {
          await cipherMutations.updateCipher.mutateAsync({
            cipherId: cipher.id,
            cipher,
          });
          toast.success(t("vault.feedback.cipher.saveSuccess.title"), {
            description: t("vault.feedback.cipher.saveSuccess.description", {
              name: cipherName,
            }),
          });
        }
        dialogState.closeCipherFormDialog();
      } catch (error) {
        const errorKey =
          dialogState.cipherFormMode === "create"
            ? "vault.feedback.cipher.createError"
            : "vault.feedback.cipher.saveError";
        toast.error(t(`${errorKey}.title`), {
          description:
            error instanceof Error
              ? error.message
              : t(`${errorKey}.description`),
        });
      }
    },
    [dialogState, cipherMutations, t],
  );

  const handleDeleteCipherConfirm = useCallback(async () => {
    if (!dialogState.selectedCipherIdForDelete) return;

    try {
      await cipherMutations.deleteCipher.mutateAsync(
        dialogState.selectedCipherIdForDelete,
      );
      toast.success(t("vault.feedback.cipher.deleteSuccess.title"), {
        description: t("vault.feedback.cipher.deleteSuccess.description", {
          name: dialogState.selectedCipherNameForDelete,
        }),
      });
      dialogState.closeDeleteCipherDialog();
    } catch (error) {
      toast.error(t("vault.feedback.cipher.deleteError.title"), {
        description:
          error instanceof Error
            ? error.message
            : t("vault.feedback.cipher.deleteError.description"),
      });
    }
  }, [dialogState, cipherMutations, t]);

  return {
    // Dialog state
    cipherFormMode: dialogState.cipherFormMode,
    isCipherFormOpen: dialogState.isCipherFormOpen,
    isDeleteCipherDialogOpen: dialogState.isDeleteCipherDialogOpen,
    selectedCipherForEdit: dialogState.selectedCipherForEdit,
    selectedCipherIdForDelete: dialogState.selectedCipherIdForDelete,
    selectedCipherNameForDelete: dialogState.selectedCipherNameForDelete,

    // Mutation state
    isCreating: cipherMutations.createCipher.isPending,
    isUpdating: cipherMutations.updateCipher.isPending,
    isDeleting: cipherMutations.deleteCipher.isPending,

    // Operation handlers
    handleCreateCipher,
    handleEditCipher,
    handleDeleteCipher,
    handleCloneCipher,
    handleCipherFormConfirm,
    handleDeleteCipherConfirm,

    // Dialog close handlers
    closeCipherFormDialog: dialogState.closeCipherFormDialog,
    closeDeleteCipherDialog: dialogState.closeDeleteCipherDialog,
  };
}
