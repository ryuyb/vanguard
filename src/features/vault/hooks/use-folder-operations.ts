import { useCallback } from "react";
import { appI18n } from "@/i18n";
import { toast } from "@/lib/toast";
import { useFolderActions } from "./use-folder-actions";
import {
  type FolderDialogMode,
  useFolderDialogState,
} from "./use-folder-dialog-state";

export type FolderOperations = {
  // 对话框状态（来自 useFolderDialogState）
  folderDialogMode: FolderDialogMode;
  isFolderDialogOpen: boolean;
  isDeleteDialogOpen: boolean;
  selectedFolderId: string | null;
  selectedFolderName: string;
  parentFolderName: string | null;

  // Mutation 状态（来自 useFolderActions）
  isCreating: boolean;
  isRenaming: boolean;
  isDeleting: boolean;

  // 操作函数
  handleCreateFolder: () => void;
  handleCreateSubFolder: (parentName: string) => void;
  handleRenameFolder: (folderId: string, currentName: string) => void;
  handleDeleteFolder: (folderId: string, folderName: string) => void;
  handleFolderDialogConfirm: (name: string) => Promise<void>;
  handleDeleteDialogConfirm: () => Promise<void>;

  // Dialog close handlers (for onOpenChange)
  closeFolderDialog: () => void;
  closeDeleteDialog: () => void;
};

export function useFolderOperations(): FolderOperations {
  const dialogState = useFolderDialogState();
  const folderActions = useFolderActions();

  // 操作函数：打开创建文件夹对话框
  const handleCreateFolder = useCallback(() => {
    dialogState.openCreateFolderDialog();
  }, [dialogState]);

  // 操作函数：打开创建子文件夹对话框
  const handleCreateSubFolder = useCallback(
    (parentName: string) => {
      dialogState.openCreateSubFolderDialog(parentName);
    },
    [dialogState],
  );

  // 操作函数：打开重命名文件夹对话框
  const handleRenameFolder = useCallback(
    (folderId: string, currentName: string) => {
      dialogState.openRenameFolderDialog(folderId, currentName);
    },
    [dialogState],
  );

  // 操作函数：打开删除文件夹对话框
  const handleDeleteFolder = useCallback(
    (folderId: string, folderName: string) => {
      dialogState.openDeleteFolderDialog(folderId, folderName);
    },
    [dialogState],
  );

  // 对话框确认：创建或重命名文件夹
  const handleFolderDialogConfirm = useCallback(
    async (name: string) => {
      if (dialogState.folderDialogMode === "create") {
        folderActions.createFolder.mutate(name, {
          onSuccess: () => {
            toast.success(
              appI18n.t("vault.feedback.folder.createSuccess.title"),
              {
                description: appI18n.t(
                  "vault.feedback.folder.createSuccess.description",
                  { name },
                ),
              },
            );
            dialogState.closeFolderDialog();
          },
          onError: () => {
            toast.error(appI18n.t("vault.feedback.folder.createError.title"), {
              description: appI18n.t(
                "vault.feedback.folder.createError.description",
              ),
            });
          },
        });
      } else if (dialogState.folderDialogMode === "rename") {
        if (!dialogState.selectedFolderId) {
          return;
        }
        folderActions.renameFolder.mutate(
          {
            folderId: dialogState.selectedFolderId,
            newName: name,
          },
          {
            onSuccess: () => {
              toast.success(
                appI18n.t("vault.feedback.folder.renameSuccess.title"),
                {
                  description: appI18n.t(
                    "vault.feedback.folder.renameSuccess.description",
                    { name },
                  ),
                },
              );
              dialogState.closeFolderDialog();
            },
            onError: () => {
              toast.error(
                appI18n.t("vault.feedback.folder.renameError.title"),
                {
                  description: appI18n.t(
                    "vault.feedback.folder.renameError.description",
                  ),
                },
              );
            },
          },
        );
      }
    },
    [dialogState, folderActions],
  );

  // 对话框确认：删除文件夹
  const handleDeleteDialogConfirm = useCallback(async () => {
    if (!dialogState.selectedFolderId) {
      return;
    }
    const folderName = dialogState.selectedFolderName;
    folderActions.deleteFolder.mutate(dialogState.selectedFolderId, {
      onSuccess: () => {
        toast.success(appI18n.t("vault.feedback.folder.deleteSuccess.title"), {
          description: appI18n.t(
            "vault.feedback.folder.deleteSuccess.description",
            { name: folderName },
          ),
        });
        dialogState.closeDeleteDialog();
      },
      onError: () => {
        toast.error(appI18n.t("vault.feedback.folder.deleteError.title"), {
          description: appI18n.t(
            "vault.feedback.folder.deleteError.description",
          ),
        });
      },
    });
  }, [dialogState, folderActions]);

  return {
    // 对话框状态
    folderDialogMode: dialogState.folderDialogMode,
    isFolderDialogOpen: dialogState.isFolderDialogOpen,
    isDeleteDialogOpen: dialogState.isDeleteDialogOpen,
    selectedFolderId: dialogState.selectedFolderId,
    selectedFolderName: dialogState.selectedFolderName,
    parentFolderName: dialogState.parentFolderName,

    // Mutation 状态
    isCreating: folderActions.createFolder.isPending,
    isRenaming: folderActions.renameFolder.isPending,
    isDeleting: folderActions.deleteFolder.isPending,

    // 操作函数
    handleCreateFolder,
    handleCreateSubFolder,
    handleRenameFolder,
    handleDeleteFolder,
    handleFolderDialogConfirm,
    handleDeleteDialogConfirm,

    // Dialog close handlers
    closeFolderDialog: dialogState.closeFolderDialog,
    closeDeleteDialog: dialogState.closeDeleteDialog,
  };
}
