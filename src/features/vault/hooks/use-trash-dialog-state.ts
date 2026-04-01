import { useState } from "react";

export type TrashDialogState = {
  // State
  isRestoreDialogOpen: boolean;
  isPermanentDeleteDialogOpen: boolean;
  trashActionCipherId: string | null;
  trashActionCipherName: string;
  isTrashActionLoading: boolean;

  // Actions
  openRestoreDialog: (cipherId: string, cipherName: string) => void;
  openPermanentDeleteDialog: (cipherId: string, cipherName: string) => void;
  closeRestoreDialog: () => void;
  closePermanentDeleteDialog: () => void;
  setIsTrashActionLoading: (loading: boolean) => void;
};

export function useTrashDialogState(): TrashDialogState {
  const [isRestoreDialogOpen, setIsRestoreDialogOpen] = useState(false);
  const [isPermanentDeleteDialogOpen, setIsPermanentDeleteDialogOpen] =
    useState(false);
  const [trashActionCipherId, setTrashActionCipherId] = useState<string | null>(
    null,
  );
  const [trashActionCipherName, setTrashActionCipherName] = useState("");
  const [isTrashActionLoading, setIsTrashActionLoading] = useState(false);

  const openRestoreDialog = (cipherId: string, cipherName: string) => {
    setTrashActionCipherId(cipherId);
    setTrashActionCipherName(cipherName);
    setIsRestoreDialogOpen(true);
  };

  const openPermanentDeleteDialog = (cipherId: string, cipherName: string) => {
    setTrashActionCipherId(cipherId);
    setTrashActionCipherName(cipherName);
    setIsPermanentDeleteDialogOpen(true);
  };

  const closeRestoreDialog = () => {
    setIsRestoreDialogOpen(false);
  };

  const closePermanentDeleteDialog = () => {
    setIsPermanentDeleteDialogOpen(false);
  };

  return {
    // State
    isRestoreDialogOpen,
    isPermanentDeleteDialogOpen,
    trashActionCipherId,
    trashActionCipherName,
    isTrashActionLoading,

    // Actions
    openRestoreDialog,
    openPermanentDeleteDialog,
    closeRestoreDialog,
    closePermanentDeleteDialog,
    setIsTrashActionLoading,
  };
}
