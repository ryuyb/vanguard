import { useEffect, useRef, useState } from "react";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { cn } from "@/lib/utils";

type TruncatableTextProps = {
  text: string;
  className?: string;
  as?: "div" | "h2" | "span" | "p";
  tooltipSide?: "top" | "bottom" | "left" | "right";
};

export function TruncatableText({
  text,
  className,
  as = "div",
  tooltipSide = "bottom",
}: TruncatableTextProps) {
  const ref = useRef<HTMLElement>(null);
  const [isTruncated, setIsTruncated] = useState(false);

  // biome-ignore lint/correctness/useExhaustiveDependencies: text change should re-check truncation
  useEffect(() => {
    const el = ref.current;
    if (!el) return;

    const checkTruncation = () => {
      setIsTruncated(el.scrollWidth > el.clientWidth);
    };

    checkTruncation();

    const resizeObserver = new ResizeObserver(checkTruncation);
    resizeObserver.observe(el.parentElement ?? el);

    return () => resizeObserver.disconnect();
  }, [text]);

  const Element = as;

  return (
    <Tooltip open={isTruncated ? undefined : false}>
      <TooltipTrigger asChild>
        <Element
          ref={ref as React.RefObject<HTMLDivElement>}
          className={cn("truncate", className)}
        >
          {text}
        </Element>
      </TooltipTrigger>
      <TooltipContent side={tooltipSide}>{text}</TooltipContent>
    </Tooltip>
  );
}
