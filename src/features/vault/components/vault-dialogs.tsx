import type { SyncCipher, VaultFolderItemDto } from "@/bindings";
import { CipherFormDialog } from "@/features/vault/components/cipher-form-dialog";
import { DeleteCipherDialog } from "@/features/vault/components/delete-cipher-dialog";
import { DeleteFolderDialog } from "@/features/vault/components/delete-folder-dialog";
import { FolderDialog } from "@/features/vault/components/folder-dialog";
import { PermanentDeleteCipherDialog } from "@/features/vault/components/permanent-delete-cipher-dialog";
import { RestoreCipherDialog } from "@/features/vault/components/restore-cipher-dialog";
import { VaultSettingsDialog } from "@/features/vault/components/vault-settings-dialog";

export type VaultDialogsProps = {
  // Settings
  isSettingsDialogOpen: boolean;
  onSettingsDialogOpenChange: (open: boolean) => void;

  // Folder operations
  folderDialogMode: "create" | "rename";
  isFolderDialogOpen: boolean;
  isDeleteFolderDialogOpen: boolean;
  selectedFolderName: string;
  parentFolderName: string | null;
  onFolderDialogOpenChange: (open: boolean) => void;
  onDeleteFolderDialogOpenChange: (open: boolean) => void;
  onFolderDialogConfirm: (name: string) => void;
  onDeleteFolderDialogConfirm: () => void;
  isFolderLoading: boolean;

  // Cipher operations
  cipherFormMode: "create" | "edit";
  isCipherFormOpen: boolean;
  isDeleteCipherDialogOpen: boolean;
  selectedCipherForEdit: SyncCipher | null;
  selectedCipherNameForDelete: string;
  cipherFolderId: string | null;
  folders: VaultFolderItemDto[];
  onCipherFormOpenChange: (open: boolean) => void;
  onDeleteCipherDialogOpenChange: (open: boolean) => void;
  onCipherFormConfirm: (cipher: SyncCipher) => void;
  onDeleteCipherDialogConfirm: () => void;
  isCipherFormLoading: boolean;
  isDeleteCipherLoading: boolean;

  // Trash operations
  isRestoreDialogOpen: boolean;
  isPermanentDeleteDialogOpen: boolean;
  trashActionCipherName: string;
  onRestoreDialogOpenChange: (open: boolean) => void;
  onPermanentDeleteDialogOpenChange: (open: boolean) => void;
  onRestoreDialogConfirm: () => void;
  onPermanentDeleteDialogConfirm: () => void;
  isTrashActionLoading: boolean;
};

export function VaultDialogs({
  // Settings
  isSettingsDialogOpen,
  onSettingsDialogOpenChange,

  // Folder operations
  folderDialogMode,
  isFolderDialogOpen,
  isDeleteFolderDialogOpen,
  selectedFolderName,
  parentFolderName,
  onFolderDialogOpenChange,
  onDeleteFolderDialogOpenChange,
  onFolderDialogConfirm,
  onDeleteFolderDialogConfirm,
  isFolderLoading,

  // Cipher operations
  cipherFormMode,
  isCipherFormOpen,
  isDeleteCipherDialogOpen,
  selectedCipherForEdit,
  selectedCipherNameForDelete,
  cipherFolderId,
  folders,
  onCipherFormOpenChange,
  onDeleteCipherDialogOpenChange,
  onCipherFormConfirm,
  onDeleteCipherDialogConfirm,
  isCipherFormLoading,
  isDeleteCipherLoading,

  // Trash operations
  isRestoreDialogOpen,
  isPermanentDeleteDialogOpen,
  trashActionCipherName,
  onRestoreDialogOpenChange,
  onPermanentDeleteDialogOpenChange,
  onRestoreDialogConfirm,
  onPermanentDeleteDialogConfirm,
  isTrashActionLoading,
}: VaultDialogsProps) {
  return (
    <>
      <VaultSettingsDialog
        open={isSettingsDialogOpen}
        onOpenChange={onSettingsDialogOpenChange}
      />

      <FolderDialog
        open={isFolderDialogOpen}
        mode={folderDialogMode}
        initialName={selectedFolderName}
        parentFolderName={parentFolderName}
        onOpenChange={onFolderDialogOpenChange}
        onConfirm={onFolderDialogConfirm}
        isLoading={isFolderLoading}
      />

      <DeleteFolderDialog
        open={isDeleteFolderDialogOpen}
        folderName={selectedFolderName}
        onOpenChange={onDeleteFolderDialogOpenChange}
        onConfirm={onDeleteFolderDialogConfirm}
        isLoading={isFolderLoading}
      />

      <CipherFormDialog
        open={isCipherFormOpen}
        mode={cipherFormMode}
        initialCipher={selectedCipherForEdit}
        folderId={cipherFolderId}
        folders={folders}
        onOpenChange={onCipherFormOpenChange}
        onConfirm={onCipherFormConfirm}
        isLoading={isCipherFormLoading}
      />

      <DeleteCipherDialog
        open={isDeleteCipherDialogOpen}
        cipherName={selectedCipherNameForDelete}
        onOpenChange={onDeleteCipherDialogOpenChange}
        onConfirm={onDeleteCipherDialogConfirm}
        isLoading={isDeleteCipherLoading}
      />

      <RestoreCipherDialog
        open={isRestoreDialogOpen}
        cipherName={trashActionCipherName}
        onOpenChange={onRestoreDialogOpenChange}
        onConfirm={onRestoreDialogConfirm}
        isLoading={isTrashActionLoading}
      />

      <PermanentDeleteCipherDialog
        open={isPermanentDeleteDialogOpen}
        cipherName={trashActionCipherName}
        onOpenChange={onPermanentDeleteDialogOpenChange}
        onConfirm={onPermanentDeleteDialogConfirm}
        isLoading={isTrashActionLoading}
      />
    </>
  );
}
