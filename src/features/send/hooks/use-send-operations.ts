import { useTranslation } from "react-i18next";
import type { SendItemDto, SyncSend } from "@/bindings";
import { toast } from "@/lib/toast";
import { useSendDialogState } from "./use-send-dialog-state";
import { useSendMutations } from "./use-send-mutations";

export type SendOperations = {
  // Dialog state
  selectedSendId: string | null;
  isSendFormOpen: boolean;
  sendFormMode: "create" | "edit";
  selectedSendForEdit: SendItemDto | null;
  isDeleteSendDialogOpen: boolean;
  selectedSendIdForDelete: string | null;
  selectedSendNameForDelete: string;

  // Mutation state
  isCreating: boolean;
  isUpdating: boolean;
  isDeleting: boolean;

  // Operations
  handleCreateSend: () => void;
  handleEditSend: (send: SendItemDto) => void;
  handleDeleteSend: (sendId: string, sendName: string) => void;
  handleSendFormConfirm: (
    send: SyncSend,
    fileData?: number[] | null,
  ) => Promise<void>;
  handleDeleteSendConfirm: () => Promise<void>;
  setSelectedSendId: (id: string | null) => void;

  // Dialog close handlers (for onOpenChange)
  closeSendFormDialog: () => void;
  closeDeleteSendDialog: () => void;
};

export function useSendOperations(): SendOperations {
  const { t } = useTranslation();
  const dialogState = useSendDialogState();
  const mutations = useSendMutations();

  const handleCreateSend = () => {
    dialogState.openCreateSendDialog();
  };

  const handleEditSend = (send: SendItemDto) => {
    dialogState.openEditSendDialog(send);
  };

  const handleDeleteSend = (sendId: string, sendName: string) => {
    dialogState.openDeleteSendDialog(sendId, sendName);
  };

  const handleSendFormConfirm = async (
    send: SyncSend,
    fileData?: number[] | null,
  ) => {
    try {
      if (dialogState.sendFormMode === "create") {
        await mutations.createSend.mutateAsync({ send, fileData });
        toast.success(t("send.feedback.createSuccess"));
      } else {
        await mutations.updateSend.mutateAsync({ sendId: send.id, send });
        toast.success(t("send.feedback.saveSuccess"));
      }
      dialogState.closeSendFormDialog();
    } catch {
      toast.error(
        dialogState.sendFormMode === "create"
          ? t("send.feedback.createError")
          : t("send.feedback.saveError"),
      );
    }
  };

  const handleDeleteSendConfirm = async () => {
    if (!dialogState.selectedSendIdForDelete) return;

    try {
      await mutations.deleteSend.mutateAsync(
        dialogState.selectedSendIdForDelete,
      );
      toast.success(t("send.feedback.deleteSuccess"));
      dialogState.closeDeleteSendDialog();
    } catch {
      toast.error(t("send.feedback.deleteError"));
    }
  };

  return {
    // Dialog state
    selectedSendId: dialogState.selectedSendId,
    isSendFormOpen: dialogState.isSendFormOpen,
    sendFormMode: dialogState.sendFormMode,
    selectedSendForEdit: dialogState.selectedSendForEdit,
    isDeleteSendDialogOpen: dialogState.isDeleteSendDialogOpen,
    selectedSendIdForDelete: dialogState.selectedSendIdForDelete,
    selectedSendNameForDelete: dialogState.selectedSendNameForDelete,

    // Mutation state
    isCreating: mutations.createSend.isPending,
    isUpdating: mutations.updateSend.isPending,
    isDeleting: mutations.deleteSend.isPending,

    // Operations
    handleCreateSend,
    handleEditSend,
    handleDeleteSend,
    handleSendFormConfirm,
    handleDeleteSendConfirm,
    setSelectedSendId: dialogState.setSelectedSendId,

    // Dialog close handlers
    closeSendFormDialog: dialogState.closeSendFormDialog,
    closeDeleteSendDialog: dialogState.closeDeleteSendDialog,
  };
}
