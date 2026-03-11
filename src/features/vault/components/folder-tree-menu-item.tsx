import { ChevronRight } from "lucide-react";
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/components/ui/collapsible";
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
};

export function FolderTreeMenuItem({
  node,
  depth,
  selectedMenuId,
  expandedNodeKeys,
  folderCipherCount,
  onFolderSelect,
  onOpenChange,
}: FolderTreeMenuItemProps) {
  const hasChildren = node.children.length > 0;
  const isExpanded = expandedNodeKeys.has(node.key);
  const isSelected = node.folderId != null && selectedMenuId === node.folderId;
  const count = countNodeCiphers(node, folderCipherCount);

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
            <span className="inline-block size-6" aria-hidden="true" />
          )}

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
                />
              ))}
            </div>
          </CollapsibleContent>
        )}
      </div>
    </Collapsible>
  );
}
