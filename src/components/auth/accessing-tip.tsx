import { useEffect, useState } from "react";
import { HugeiconsIcon } from "@hugeicons/react";
import { ArrowRight01Icon } from "@hugeicons/core-free-icons";

import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuRadioGroup,
  DropdownMenuRadioItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import {
  SelfHostedModal,
  type SelfHostedConfig,
} from "@/components/auth/self-hosted-modal";
import { appStore } from "@/lib/tauri-store";

const HOST_OPTIONS = ["bitwarden.com", "bitwarden.eu", "self-hosted"] as const;
type HostOption = (typeof HOST_OPTIONS)[number];

type AccessingTipProps = {
  className?: string;
  defaultHost?: HostOption;
  onChange?: (value: HostOption) => void;
};

export function AccessingTip({
  className,
  defaultHost = "bitwarden.com",
  onChange,
}: AccessingTipProps) {
  const [accessHost, setAccessHost] = useState<HostOption>(defaultHost);
  const [selfHostedOpen, setSelfHostedOpen] = useState(false);
  const [selfHostedConfig, setSelfHostedConfig] = useState<SelfHostedConfig>({
    serverUrl: "",
  });

  useEffect(() => {
    let active = true;

    const loadFromStore = async () => {
      const [serverHost, selfHosted] = await Promise.all([
        appStore.get("serverHost"),
        appStore.get("selfHosted"),
      ]);

      if (!active) {
        return;
      }

      if (serverHost && HOST_OPTIONS.includes(serverHost)) {
        setAccessHost(serverHost);
      }

      if (selfHosted) {
        setSelfHostedConfig(selfHosted);
      }
    };

    void loadFromStore();

    return () => {
      active = false;
    };
  }, []);

  return (
    <div className={className}>
      Accessing:{" "}
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <button
            type="button"
            className="text-primary inline-flex items-center gap-1 font-medium underline-offset-4 hover:underline"
          >
            {accessHost}
            <HugeiconsIcon icon={ArrowRight01Icon} size={14} />
          </button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="center" sideOffset={8} className="w-36">
          <DropdownMenuRadioGroup
            value={accessHost}
            onValueChange={(value) => {
              if (!HOST_OPTIONS.includes(value as HostOption)) {
                return;
              }
              void appStore.set("serverHost", value as HostOption);
              setAccessHost(value as HostOption);
              onChange?.(value as HostOption);
              if (value === "self-hosted") {
                setSelfHostedOpen(true);
              }
            }}
          >
            {HOST_OPTIONS.map((option) => (
              <DropdownMenuRadioItem key={option} value={option}>
                {option}
              </DropdownMenuRadioItem>
            ))}
          </DropdownMenuRadioGroup>
        </DropdownMenuContent>
      </DropdownMenu>
      <SelfHostedModal
        open={selfHostedOpen}
        onOpenChange={setSelfHostedOpen}
        value={selfHostedConfig}
        onSave={async (value) => {
          setSelfHostedConfig(value);
          await appStore.set("selfHosted", value);
        }}
      />
    </div>
  );
}
