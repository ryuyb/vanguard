import { useCallback, useState } from "react";

export type FolderDialogMode = "create" | "rename";

export type FolderDialogState = {
  // State
  folderDialogMode: FolderDialogMode;
  isFolderDialogOpen: boolean;
  isDeleteDialogOpen: boolean;
  selectedFolderId: string | null;
  selectedFolderName: string;
  parentFolderName: string | null;

  // Actions
  openCreateFolderDialog: () => void;
  openCreateSubFolderDialog: (parentName: string) => void;
  openRenameFolderDialog: (folderId: string, currentName: string) => void;
  openDeleteFolderDialog: (folderId: string, folderName: string) => void;
  closeFolderDialog: () => void;
  closeDeleteDialog: () => void;
};

export function useFolderDialogState(): FolderDialogState {
  const [folderDialogMode, setFolderDialogMode] =
    useState<FolderDialogMode>("create");
  const [isFolderDialogOpen, setIsFolderDialogOpen] = useState(false);
  const [isDeleteDialogOpen, setIsDeleteDialogOpen] = useState(false);
  const [selectedFolderId, setSelectedFolderId] = useState<string | null>(null);
  const [selectedFolderName, setSelectedFolderName] = useState("");
  const [parentFolderName, setParentFolderName] = useState<string | null>(null);

  const openCreateFolderDialog = useCallback(() => {
    setFolderDialogMode("create");
    setSelectedFolderName("");
    setParentFolderName(null);
    setIsFolderDialogOpen(true);
  }, []);

  const openCreateSubFolderDialog = useCallback((parentName: string) => {
    setFolderDialogMode("create");
    setSelectedFolderName("");
    setParentFolderName(parentName);
    setIsFolderDialogOpen(true);
  }, []);

  const openRenameFolderDialog = useCallback(
    (folderId: string, currentName: string) => {
      setFolderDialogMode("rename");
      setSelectedFolderId(folderId);
      setSelectedFolderName(currentName);
      setParentFolderName(null);
      setIsFolderDialogOpen(true);
    },
    [],
  );

  const openDeleteFolderDialog = useCallback(
    (folderId: string, folderName: string) => {
      setSelectedFolderId(folderId);
      setSelectedFolderName(folderName);
      setIsDeleteDialogOpen(true);
    },
    [],
  );

  const closeFolderDialog = useCallback(() => {
    setIsFolderDialogOpen(false);
  }, []);

  const closeDeleteDialog = useCallback(() => {
    setIsDeleteDialogOpen(false);
  }, []);

  return {
    // State
    folderDialogMode,
    isFolderDialogOpen,
    isDeleteDialogOpen,
    selectedFolderId,
    selectedFolderName,
    parentFolderName,

    // Actions
    openCreateFolderDialog,
    openCreateSubFolderDialog,
    openRenameFolderDialog,
    openDeleteFolderDialog,
    closeFolderDialog,
    closeDeleteDialog,
  };
}
