import { useCallback, useMemo, useState } from "react";
import { commands, type SendItemDto } from "@/bindings";
import type { SendTypeFilter } from "../types";

export function useSendList() {
  const [sends, setSends] = useState<SendItemDto[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [sendTypeFilter, setSendTypeFilter] = useState<SendTypeFilter>("all");
  const [searchQuery, setSearchQuery] = useState("");

  const reload = useCallback(async () => {
    setIsLoading(true);
    try {
      const result = await commands.listSends();
      if (result.status === "ok") {
        setSends(result.data);
      }
    } finally {
      setIsLoading(false);
    }
  }, []);

  const filteredSends = useMemo(() => {
    return sends.filter((send) => {
      if (sendTypeFilter === "text" && send.type !== 0) return false;
      if (sendTypeFilter === "file" && send.type !== 1) return false;
      if (searchQuery.trim()) {
        const q = searchQuery.toLowerCase();
        return send.name?.toLowerCase().includes(q) ?? false;
      }
      return true;
    });
  }, [sends, sendTypeFilter, searchQuery]);

  return {
    filteredSends,
    sendCount: sends.length,
    isLoading,
    sendTypeFilter,
    setSendTypeFilter,
    searchQuery,
    setSearchQuery,
    reload,
  };
}
