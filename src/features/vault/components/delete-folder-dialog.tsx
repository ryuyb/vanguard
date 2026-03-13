import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";
import { appI18n } from "@/i18n";

type DeleteFolderDialogProps = {
  open: boolean;
  folderName: string;
  onOpenChange: (open: boolean) => void;
  onConfirm: () => void;
  isLoading?: boolean;
};

export function DeleteFolderDialog({
  open,
  folderName,
  onOpenChange,
  onConfirm,
  isLoading = false,
}: DeleteFolderDialogProps) {
  return (
    <AlertDialog open={open} onOpenChange={onOpenChange}>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>
            {appI18n.t("vault.dialogs.deleteFolder.title")}
          </AlertDialogTitle>
          <AlertDialogDescription>
            {appI18n.t("vault.dialogs.deleteFolder.descriptionPrefix")}{" "}
            <strong className="text-slate-900">"{folderName}"</strong>{" "}
            {appI18n.t("vault.dialogs.deleteFolder.descriptionSuffix")}
            <br />
            <br />
            {appI18n.t("vault.dialogs.deleteFolder.descriptionHint")}
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter className="gap-2">
          <AlertDialogCancel disabled={isLoading}>
            {appI18n.t("common.actions.cancel")}
          </AlertDialogCancel>
          <AlertDialogAction
            onClick={(e) => {
              e.preventDefault();
              onConfirm();
            }}
            disabled={isLoading}
            className="bg-red-600 hover:bg-red-700"
          >
            {isLoading
              ? appI18n.t("vault.dialogs.deleteFolder.deleting")
              : appI18n.t("vault.page.actions.delete")}
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}
