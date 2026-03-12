import { useMutation } from "@tanstack/react-query";
import { useState } from "react";
import { commands } from "@/bindings";

type UseFolderActionsOptions = {
  onSuccess?: () => void;
};

export function useFolderActions(options?: UseFolderActionsOptions) {
  const [isCreating, setIsCreating] = useState(false);
  const [isRenaming, setIsRenaming] = useState(false);
  const [isDeleting, setIsDeleting] = useState(false);

  const createFolderMutation = useMutation({
    mutationFn: async (name: string) => {
      const result = await commands.createFolder({ name });
      if (result.status === "error") {
        throw new Error(result.error.message);
      }
      return result.data;
    },
    onSuccess: () => {
      options?.onSuccess?.();
    },
  });

  const renameFolderMutation = useMutation({
    mutationFn: async ({
      folderId,
      newName,
    }: {
      folderId: string;
      newName: string;
    }) => {
      const result = await commands.renameFolder({ folderId, newName });
      if (result.status === "error") {
        throw new Error(result.error.message);
      }
      return result.data;
    },
    onSuccess: () => {
      options?.onSuccess?.();
    },
  });

  const deleteFolderMutation = useMutation({
    mutationFn: async (folderId: string) => {
      const result = await commands.deleteFolder({ folderId });
      if (result.status === "error") {
        throw new Error(result.error.message);
      }
      return result.data;
    },
    onSuccess: () => {
      options?.onSuccess?.();
    },
  });

  return {
    createFolder: {
      mutate: createFolderMutation.mutate,
      isLoading: createFolderMutation.isPending,
      error: createFolderMutation.error,
    },
    renameFolder: {
      mutate: renameFolderMutation.mutate,
      isLoading: renameFolderMutation.isPending,
      error: renameFolderMutation.error,
    },
    deleteFolder: {
      mutate: deleteFolderMutation.mutate,
      isLoading: deleteFolderMutation.isPending,
      error: deleteFolderMutation.error,
    },
    isCreating,
    setIsCreating,
    isRenaming,
    setIsRenaming,
    isDeleting,
    setIsDeleting,
  };
}
