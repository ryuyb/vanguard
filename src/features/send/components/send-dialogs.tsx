import type { SendItemDto, SyncSend } from "@/bindings";
import { DeleteSendDialog } from "./delete-send-dialog";
import { SendFormDialog } from "./send-form-dialog";

type SendDialogsProps = {
  // Dialog state
  isSendFormOpen: boolean;
  sendFormMode: "create" | "edit";
  selectedSendForEdit: SendItemDto | null;
  isDeleteSendDialogOpen: boolean;
  selectedSendNameForDelete: string;

  // Dialog controls
  onSendFormOpenChange: (open: boolean) => void;
  onDeleteSendOpenChange: (open: boolean) => void;

  // Confirm handlers
  onSendFormConfirm: (
    send: SyncSend,
    fileData?: number[] | null,
  ) => Promise<void>;
  onDeleteSendConfirm: () => Promise<void>;
  onRemovePassword: (sendId: string) => Promise<void>;

  // Loading states
  isSendFormLoading: boolean;
  isDeleteSendLoading: boolean;
};

export function SendDialogs({
  isSendFormOpen,
  sendFormMode,
  selectedSendForEdit,
  isDeleteSendDialogOpen,
  selectedSendNameForDelete,
  onSendFormOpenChange,
  onDeleteSendOpenChange,
  onSendFormConfirm,
  onDeleteSendConfirm,
  onRemovePassword,
  isSendFormLoading,
  isDeleteSendLoading,
}: SendDialogsProps) {
  return (
    <>
      <SendFormDialog
        key={
          isSendFormOpen
            ? `send-form-${selectedSendForEdit?.id ?? "new"}`
            : "closed"
        }
        open={isSendFormOpen}
        mode={sendFormMode}
        initialSend={selectedSendForEdit}
        onOpenChange={onSendFormOpenChange}
        onConfirm={(send, fileData) => void onSendFormConfirm(send, fileData)}
        onRemovePassword={onRemovePassword}
        isLoading={isSendFormLoading}
      />

      <DeleteSendDialog
        open={isDeleteSendDialogOpen}
        sendName={selectedSendNameForDelete}
        onOpenChange={onDeleteSendOpenChange}
        onConfirm={() => void onDeleteSendConfirm()}
        isLoading={isDeleteSendLoading}
      />
    </>
  );
}
