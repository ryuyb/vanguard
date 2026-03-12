import { ChevronRight, Edit2, Folder, FolderPlus, Trash2 } from "lucide-react";
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/components/ui/collapsible";
import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuSeparator,
  ContextMenuTrigger,
} from "@/components/ui/context-menu";
import type { FolderTreeNode } from "@/features/vault/types";
import { countNodeCiphers } from "@/features/vault/utils";

type FolderTreeMenuItemProps = {
  node: FolderTreeNode;
  depth: number;
  selectedMenuId: string;
  expandedNodeKeys: Set<string>;
  folderCipherCount: Map<string, number>;
  onFolderSelect: (folderId: string) => void;
  onOpenChange: (nodeKey: string, open: boolean) => void;
  onRenameFolder?: (folderId: string, currentName: string) => void;
  onDeleteFolder?: (folderId: string, folderName: string) => void;
  onCreateSubFolder?: (parentFolderPath: string) => void;
};

export function FolderTreeMenuItem({
  node,
  depth,
  selectedMenuId,
  expandedNodeKeys,
  folderCipherCount,
  onFolderSelect,
  onOpenChange,
  onRenameFolder,
  onDeleteFolder,
  onCreateSubFolder,
}: FolderTreeMenuItemProps) {
  const hasChildren = node.children.length > 0;
  const isExpanded = expandedNodeKeys.has(node.key);
  const isSelected = node.folderId != null && selectedMenuId === node.folderId;
  const count = countNodeCiphers(node, folderCipherCount);

  const folderButton = (
    <button
      type="button"
      onClick={() => {
        if (node.folderId) {
          onFolderSelect(node.folderId);
          return;
        }
        if (hasChildren) {
          onOpenChange(node.key, !isExpanded);
        }
      }}
      className={[
        "flex min-w-0 flex-1 items-center justify-between gap-2 rounded-lg px-2 py-1.5 text-left text-sm font-medium transition-all",
        isSelected
          ? "bg-blue-50 text-blue-700 shadow-sm"
          : "text-slate-700 hover:bg-slate-50",
      ].join(" ")}
    >
      <span className="truncate">{node.label}</span>
      <span
        className={[
          "text-xs font-semibold",
          isSelected ? "text-blue-600" : "text-slate-400",
        ].join(" ")}
      >
        {count}
      </span>
    </button>
  );

  return (
    <Collapsible
      open={hasChildren ? isExpanded : false}
      onOpenChange={(open) => onOpenChange(node.key, open)}
    >
      <div className="space-y-1" style={{ paddingLeft: `${depth * 10}px` }}>
        <div className="flex items-center gap-1">
          {hasChildren ? (
            <CollapsibleTrigger asChild>
              <button
                type="button"
                className="flex size-6 items-center justify-center rounded text-slate-500 hover:bg-slate-100"
                aria-label={isExpanded ? "collapse folder" : "expand folder"}
              >
                <ChevronRight
                  className={[
                    "size-4 transition-transform",
                    isExpanded ? "rotate-90" : "",
                  ].join(" ")}
                />
              </button>
            </CollapsibleTrigger>
          ) : (
            <span className="inline-flex size-6 items-center justify-center" aria-hidden="true">
              <Folder className="size-4 text-slate-400" />
            </span>
          )}

          {node.folderId &&
          onRenameFolder &&
          onDeleteFolder &&
          onCreateSubFolder ? (
            <ContextMenu>
              <ContextMenuTrigger asChild>{folderButton}</ContextMenuTrigger>
              <ContextMenuContent className="w-48">
                <ContextMenuItem
                  onClick={() => onCreateSubFolder(node.key)}
                  className="gap-2"
                >
                  <FolderPlus className="size-4" />
                  <span>新建子文件夹</span>
                </ContextMenuItem>
                <ContextMenuSeparator />
                <ContextMenuItem
                  onClick={() => onRenameFolder(node.folderId!, node.label)}
                  className="gap-2"
                >
                  <Edit2 className="size-4" />
                  <span>重命名</span>
                </ContextMenuItem>
                <ContextMenuItem
                  onClick={() => onDeleteFolder(node.folderId!, node.label)}
                  className="gap-2 text-red-600 focus:text-red-600"
                >
                  <Trash2 className="size-4" />
                  <span>删除</span>
                </ContextMenuItem>
              </ContextMenuContent>
            </ContextMenu>
          ) : (
            folderButton
          )}
        </div>

        {hasChildren && (
          <CollapsibleContent>
            <div className="space-y-1">
              {node.children.map((child) => (
                <FolderTreeMenuItem
                  key={child.key}
                  node={child}
                  depth={depth + 1}
                  selectedMenuId={selectedMenuId}
                  expandedNodeKeys={expandedNodeKeys}
                  folderCipherCount={folderCipherCount}
                  onFolderSelect={onFolderSelect}
                  onOpenChange={onOpenChange}
                  onRenameFolder={onRenameFolder}
                  onDeleteFolder={onDeleteFolder}
                  onCreateSubFolder={onCreateSubFolder}
                />
              ))}
            </div>
          </CollapsibleContent>
        )}
      </div>
    </Collapsible>
  );
}
