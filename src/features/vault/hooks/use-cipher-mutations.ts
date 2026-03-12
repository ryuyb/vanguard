import { useMutation } from "@tanstack/react-query";
import { commands, type SyncCipher } from "@/bindings";

type UseCipherMutationsOptions = {
  onSuccess?: () => void;
};

export function useCipherMutations(options?: UseCipherMutationsOptions) {
  const createCipherMutation = useMutation({
    mutationFn: async (cipher: SyncCipher) => {
      const result = await commands.createCipher({ cipher });
      if (result.status === "error") {
        throw new Error(result.error.message);
      }
      return result.data;
    },
    onSuccess: () => {
      options?.onSuccess?.();
    },
  });

  const updateCipherMutation = useMutation({
    mutationFn: async ({
      cipherId,
      cipher,
    }: {
      cipherId: string;
      cipher: SyncCipher;
    }) => {
      const result = await commands.updateCipher({ cipherId, cipher });
      if (result.status === "error") {
        throw new Error(result.error.message);
      }
      return result.data;
    },
    onSuccess: () => {
      options?.onSuccess?.();
    },
  });

  const deleteCipherMutation = useMutation({
    mutationFn: async (cipherId: string) => {
      const result = await commands.deleteCipher({ cipherId });
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
    createCipher: {
      mutate: createCipherMutation.mutate,
      mutateAsync: createCipherMutation.mutateAsync,
      isLoading: createCipherMutation.isPending,
      error: createCipherMutation.error,
    },
    updateCipher: {
      mutate: updateCipherMutation.mutate,
      mutateAsync: updateCipherMutation.mutateAsync,
      isLoading: updateCipherMutation.isPending,
      error: updateCipherMutation.error,
    },
    deleteCipher: {
      mutate: deleteCipherMutation.mutate,
      mutateAsync: deleteCipherMutation.mutateAsync,
      isLoading: deleteCipherMutation.isPending,
      error: deleteCipherMutation.error,
    },
  };
}
