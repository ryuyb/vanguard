import { useCallback, useState } from "react";
import {
  commands,
  type SyncCipher,
  type VaultCipherDetailDto,
} from "@/bindings";
import { vaultCipherDetailToSyncCipher } from "@/features/vault/utils/cipher-converter";

export type CipherFormMode = "create" | "edit";

export type CipherDialogState = {
  // State
  cipherFormMode: CipherFormMode;
  isCipherFormOpen: boolean;
  isDeleteCipherDialogOpen: boolean;
  selectedCipherForEdit: SyncCipher | null;
  selectedCipherIdForDelete: string | null;
  selectedCipherNameForDelete: string;

  // Actions
  openCreateCipherDialog: () => void;
  openEditCipherDialog: (cipher: VaultCipherDetailDto) => void;
  openDeleteCipherDialog: (cipherId: string, cipherName: string) => void;
  cloneCipher: (cipherId: string, cloneSuffix: string) => Promise<void>;
  closeCipherFormDialog: () => void;
  closeDeleteCipherDialog: () => void;
};

export function useCipherDialogState(): CipherDialogState {
  const [cipherFormMode, setCipherFormMode] =
    useState<CipherFormMode>("create");
  const [isCipherFormOpen, setIsCipherFormOpen] = useState(false);
  const [isDeleteCipherDialogOpen, setIsDeleteCipherDialogOpen] =
    useState(false);
  const [selectedCipherForEdit, setSelectedCipherForEdit] =
    useState<SyncCipher | null>(null);
  const [selectedCipherIdForDelete, setSelectedCipherIdForDelete] = useState<
    string | null
  >(null);
  const [selectedCipherNameForDelete, setSelectedCipherNameForDelete] =
    useState("");

  const openCreateCipherDialog = useCallback(() => {
    setCipherFormMode("create");
    setSelectedCipherForEdit(null);
    setIsCipherFormOpen(true);
  }, []);

  const openEditCipherDialog = useCallback((cipher: VaultCipherDetailDto) => {
    setCipherFormMode("edit");
    setSelectedCipherForEdit(vaultCipherDetailToSyncCipher(cipher));
    setIsCipherFormOpen(true);
  }, []);

  const openDeleteCipherDialog = useCallback(
    (cipherId: string, cipherName: string) => {
      setSelectedCipherIdForDelete(cipherId);
      setSelectedCipherNameForDelete(cipherName);
      setIsDeleteCipherDialogOpen(true);
    },
    [],
  );

  const cloneCipher = useCallback(
    async (cipherId: string, cloneSuffix: string) => {
      const result = await commands.vaultGetCipherDetail({ cipherId });
      if (result.status === "error") return;

      const cloned = vaultCipherDetailToSyncCipher(result.data.cipher);
      cloned.id = "";
      const baseName = cloned.name ?? "";
      cloned.name = `${baseName} - ${cloneSuffix}`;
      if (cloned.data) cloned.data.name = cloned.name;

      setCipherFormMode("create");
      setSelectedCipherForEdit(cloned);
      setIsCipherFormOpen(true);
    },
    [],
  );

  const closeCipherFormDialog = useCallback(() => {
    setIsCipherFormOpen(false);
  }, []);

  const closeDeleteCipherDialog = useCallback(() => {
    setIsDeleteCipherDialogOpen(false);
  }, []);

  return {
    // State
    cipherFormMode,
    isCipherFormOpen,
    isDeleteCipherDialogOpen,
    selectedCipherForEdit,
    selectedCipherIdForDelete,
    selectedCipherNameForDelete,

    // Actions
    openCreateCipherDialog,
    openEditCipherDialog,
    openDeleteCipherDialog,
    cloneCipher,
    closeCipherFormDialog,
    closeDeleteCipherDialog,
  };
}
