import { Command as CommandPrimitive } from "cmdk";
import { SearchIcon } from "lucide-react";
import type { KeyboardEvent } from "react";
import { InputGroup, InputGroupAddon } from "@/components/ui/input-group";

type SpotlightSearchInputProps = {
  query: string;
  onQueryChange: (value: string) => void;
  onKeyDown: (event: KeyboardEvent<HTMLInputElement>) => void;
};

export function SpotlightSearchInput({
  query,
  onQueryChange,
  onKeyDown,
}: SpotlightSearchInputProps) {
  return (
    <InputGroup className="h-auto rounded-xl border border-slate-200 bg-slate-50/50 px-3.5 py-3 shadow-sm transition-colors focus-within:border-blue-300 focus-within:bg-white has-[[data-slot=input-group-control]:focus-visible]:ring-0">
      <CommandPrimitive.Input
        id="spotlight-search-input"
        className="w-full bg-transparent text-base leading-[1.4] text-slate-900 outline-none placeholder:text-slate-500 focus-visible:ring-0"
        value={query}
        onValueChange={onQueryChange}
        autoFocus
        spellCheck={false}
        autoCorrect="off"
        autoCapitalize="off"
        placeholder="搜索密码库..."
        aria-label="Search"
        onKeyDown={onKeyDown}
      />
      <InputGroupAddon
        align="inline-end"
        className="size-7 min-w-7 justify-center rounded-lg bg-slate-200/60 p-0 text-slate-600"
      >
        <SearchIcon className="size-4 shrink-0" />
      </InputGroupAddon>
    </InputGroup>
  );
}
