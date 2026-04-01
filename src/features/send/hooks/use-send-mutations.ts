import { useMutation } from "@tanstack/react-query";
import { commands, type SyncSend } from "@/bindings";

type UseSendMutationsOptions = {
  onSuccess?: () => void;
};

export function useSendMutations(options?: UseSendMutationsOptions) {
  const createSendMutation = useMutation({
    mutationFn: async ({
      send,
      fileData,
    }: {
      send: SyncSend;
      fileData?: number[] | null;
    }) => {
      const result = await commands.createSend({
        send,
        fileData: fileData ?? null,
      });
      if (result.status === "error") throw new Error(result.error.message);
      return result.data;
    },
    onSuccess: () => options?.onSuccess?.(),
  });

  const updateSendMutation = useMutation({
    mutationFn: async ({
      sendId,
      send,
    }: {
      sendId: string;
      send: SyncSend;
    }) => {
      const result = await commands.updateSend({ sendId, send });
      if (result.status === "error") throw new Error(result.error.message);
      return result.data;
    },
    onSuccess: () => options?.onSuccess?.(),
  });

  const deleteSendMutation = useMutation({
    mutationFn: async (sendId: string) => {
      const result = await commands.deleteSend({ sendId });
      if (result.status === "error") throw new Error(result.error.message);
      return result.data;
    },
    onSuccess: () => options?.onSuccess?.(),
  });

  const removeSendPasswordMutation = useMutation({
    mutationFn: async (sendId: string) => {
      const result = await commands.removeSendPassword({ sendId });
      if (result.status === "error") throw new Error(result.error.message);
      return result.data;
    },
    onSuccess: () => options?.onSuccess?.(),
  });

  return {
    createSend: {
      mutateAsync: createSendMutation.mutateAsync,
      isPending: createSendMutation.isPending,
      error: createSendMutation.error,
    },
    updateSend: {
      mutateAsync: updateSendMutation.mutateAsync,
      isPending: updateSendMutation.isPending,
      error: updateSendMutation.error,
    },
    deleteSend: {
      mutateAsync: deleteSendMutation.mutateAsync,
      isPending: deleteSendMutation.isPending,
      error: deleteSendMutation.error,
    },
    removeSendPassword: {
      mutateAsync: removeSendPasswordMutation.mutateAsync,
      isPending: removeSendPasswordMutation.isPending,
      error: removeSendPasswordMutation.error,
    },
  };
}
