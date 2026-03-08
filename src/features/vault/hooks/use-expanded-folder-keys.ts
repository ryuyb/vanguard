import type { Dispatch, SetStateAction } from "react";
import { useEffect } from "react";
import type { FolderTreeNode } from "@/features/vault/types";

export function useExpandedFolderKeys({
  folderTree,
  folderTreeKeys,
  setExpandedNodeKeys,
}: {
  folderTree: FolderTreeNode[];
  folderTreeKeys: string[];
  setExpandedNodeKeys: Dispatch<SetStateAction<Set<string>>>;
}) {
  useEffect(() => {
    setExpandedNodeKeys((previous) => {
      const validKeys = new Set(folderTreeKeys);
      const next = new Set<string>();
      for (const key of previous) {
        if (validKeys.has(key)) {
          next.add(key);
        }
      }
      if (next.size === 0) {
        for (const node of folderTree) {
          next.add(node.key);
        }
      }
      return next;
    });
  }, [folderTree, folderTreeKeys, setExpandedNodeKeys]);
}
