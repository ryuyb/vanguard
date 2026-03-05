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
    <InputGroup className="h-auto rounded-[14px] border border-slate-400/45 bg-white/55 px-3 py-2.5 shadow-none transition-none has-[[data-slot=input-group-control]:focus-visible]:border-slate-400/45 has-[[data-slot=input-group-control]:focus-visible]:ring-0">
      <CommandPrimitive.Input
        id="spotlight-search-input"
        className="w-full bg-transparent text-[18px] leading-[1.3] text-slate-900 outline-none placeholder:text-slate-500 focus-visible:ring-0"
        value={query}
        onValueChange={onQueryChange}
        autoFocus
        spellCheck={false}
        autoCorrect="off"
        autoCapitalize="off"
        placeholder="Search vault ciphers by keyword"
        aria-label="Search"
        onKeyDown={onKeyDown}
      />
      <InputGroupAddon
        align="inline-end"
        className="size-6 min-w-6 justify-center rounded-[7px] bg-slate-400/20 p-0 text-slate-700"
      >
        <SearchIcon className="size-4 shrink-0" />
      </InputGroupAddon>
    </InputGroup>
  );
}
