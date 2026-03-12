import { useEffect } from "react";
import { commands, events, type FolderDto } from "@/bindings";

type UseFoldersSyncOptions = {
  onFoldersSynced?: (folders: FolderDto[]) => void;
};

export function useFoldersSync(options?: UseFoldersSyncOptions) {
  useEffect(() => {
    const unlisten = events.vaultFoldersSynced.listen(async (event) => {
      console.log(`Folders synced: ${event.payload.folderCount} folders`);

      // 调用 listFolders 获取最新的 folder 列表
      const result = await commands.listFolders();
      if (result.status === "ok") {
        options?.onFoldersSynced?.(result.data);
      }
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, [options?.onFoldersSynced]);
}
