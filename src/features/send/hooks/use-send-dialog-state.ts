import { useState } from "react";
import type { SendItemDto } from "@/bindings";

export type SendDialogState = {
  // State
  selectedSendId: string | null;
  isSendFormOpen: boolean;
  sendFormMode: "create" | "edit";
  selectedSendForEdit: SendItemDto | null;
  isDeleteSendDialogOpen: boolean;
  selectedSendIdForDelete: string | null;
  selectedSendNameForDelete: string;

  // Actions
  openCreateSendDialog: () => void;
  openEditSendDialog: (send: SendItemDto) => void;
  openDeleteSendDialog: (sendId: string, sendName: string) => void;
  closeSendFormDialog: () => void;
  closeDeleteSendDialog: () => void;
  setSelectedSendId: (id: string | null) => void;
};

export function useSendDialogState(): SendDialogState {
  const [selectedSendId, setSelectedSendId] = useState<string | null>(null);
  const [isSendFormOpen, setIsSendFormOpen] = useState(false);
  const [sendFormMode, setSendFormMode] = useState<"create" | "edit">("create");
  const [selectedSendForEdit, setSelectedSendForEdit] =
    useState<SendItemDto | null>(null);
  const [isDeleteSendDialogOpen, setIsDeleteSendDialogOpen] = useState(false);
  const [selectedSendIdForDelete, setSelectedSendIdForDelete] = useState<
    string | null
  >(null);
  const [selectedSendNameForDelete, setSelectedSendNameForDelete] =
    useState("");

  const openCreateSendDialog = () => {
    setSendFormMode("create");
    setSelectedSendForEdit(null);
    setIsSendFormOpen(true);
  };

  const openEditSendDialog = (send: SendItemDto) => {
    setSendFormMode("edit");
    setSelectedSendForEdit(send);
    setIsSendFormOpen(true);
  };

  const openDeleteSendDialog = (sendId: string, sendName: string) => {
    setSelectedSendIdForDelete(sendId);
    setSelectedSendNameForDelete(sendName);
    setIsDeleteSendDialogOpen(true);
  };

  const closeSendFormDialog = () => {
    setIsSendFormOpen(false);
  };

  const closeDeleteSendDialog = () => {
    setIsDeleteSendDialogOpen(false);
  };

  return {
    // State
    selectedSendId,
    isSendFormOpen,
    sendFormMode,
    selectedSendForEdit,
    isDeleteSendDialogOpen,
    selectedSendIdForDelete,
    selectedSendNameForDelete,

    // Actions
    openCreateSendDialog,
    openEditSendDialog,
    openDeleteSendDialog,
    closeSendFormDialog,
    closeDeleteSendDialog,
    setSelectedSendId,
  };
}
