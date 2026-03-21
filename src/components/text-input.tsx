import * as React from "react";

import { Input as BaseInput } from "@/components/ui/input";
import { cn } from "@/lib/utils";

type TextInputProps = React.ComponentProps<typeof BaseInput> & {
  inputGroup?: boolean;
};

const TextInput = React.forwardRef<
  React.ComponentRef<typeof BaseInput>,
  TextInputProps
>(
  (
    {
      autoCapitalize = "off",
      autoCorrect = "off",
      spellCheck = false,
      inputGroup = false,
      className,
      ...props
    },
    ref,
  ) => {
    const inputGroupProps = inputGroup
      ? {
          "data-slot": "input-group-control",
          className:
            "flex-1 rounded-none border-0 bg-transparent shadow-none ring-0 focus-visible:ring-0 aria-invalid:ring-0 dark:bg-transparent",
        }
      : undefined;

    return (
      <BaseInput
        ref={ref}
        autoCapitalize={autoCapitalize}
        autoCorrect={autoCorrect}
        spellCheck={spellCheck}
        className={cn(inputGroupProps?.className, className)}
        {...inputGroupProps}
        {...props}
      />
    );
  },
);

TextInput.displayName = "TextInput";

export { TextInput };
