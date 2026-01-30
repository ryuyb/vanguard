import { useForm } from "@tanstack/react-form";
import { useState } from "react";
import { z } from "zod";

import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import {
  InputGroup,
  InputGroupAddon,
  InputGroupButton,
  InputGroupInput,
} from "@/components/ui/input-group";
import { Label } from "@/components/ui/label";
import { Separator } from "@/components/ui/separator";
import { cn } from "@/lib/utils";
import welcomeBackIllustration from "@/assets/welcome-back.svg";
import { HugeiconsIcon } from "@hugeicons/react";
import {
  EyeIcon,
  LaptopCheckIcon,
  ViewOffIcon,
} from "@hugeicons/core-free-icons";

type WelcomeBackValues = {
  masterPassword: string;
};

type WelcomeBackProps = {
  className?: string;
  email?: string;
  onContinue?: (values: WelcomeBackValues) => void | Promise<void>;
  onDeviceLogin?: () => void | Promise<void>;
  onBack?: () => void;
  onRegisterClick?: () => void;
};

const passwordSchema = z
  .string()
  .min(1, { message: "Master password is required" });

function getErrorMessage(errors: unknown[] | undefined) {
  if (!errors?.length) {
    return undefined;
  }

  for (const error of errors) {
    if (!error) {
      continue;
    }

    if (typeof error === "string") {
      return error;
    }

    if (typeof error === "object" && "message" in error) {
      const message = (error as { message?: unknown }).message;
      if (typeof message === "string") {
        return message;
      }
    }
  }

  return undefined;
}

export function WelcomeBack({
  className,
  email,
  onContinue,
  onDeviceLogin,
  onBack,
  onRegisterClick,
}: WelcomeBackProps) {
  const [showPassword, setShowPassword] = useState(false);
  const defaultValues: WelcomeBackValues = {
    masterPassword: "",
  };

  const form = useForm({
    defaultValues,
    onSubmit: async ({ value }) => {
      await onContinue?.(value);
    },
  });

  const validatePassword = (value: string) => {
    const result = passwordSchema.safeParse(value);
    if (result.success) {
      return undefined;
    }
    return result.error.issues[0]?.message ?? "Master password is required";
  };

  return (
    <div className={cn("w-full max-w-md space-y-8", className)}>
      <img
        src={welcomeBackIllustration}
        alt="Welcome back"
        className="mx-auto h-20"
      />
      <h2 className="text-foreground text-center text-2xl font-semibold">
        Welcome back
      </h2>
      {email ? (
        <p className="text-muted-foreground text-center text-sm">{email}</p>
      ) : null}
      <Card className="w-full">
        <CardContent>
          <form
            className="space-y-4"
            noValidate
            onSubmit={(event) => {
              event.preventDefault();
              event.stopPropagation();
              form.handleSubmit();
            }}
          >
            <form.Field
              name="masterPassword"
              validators={{
                onBlur: ({ value }) => validatePassword(value),
                onSubmit: ({ value }) => validatePassword(value),
              }}
            >
              {(field) => {
                const errorMessage = getErrorMessage(field.state.meta.errors);

                return (
                  <div className="space-y-2">
                    <Label htmlFor={field.name}>Master password</Label>
                    <InputGroup>
                      <InputGroupInput
                        id={field.name}
                        name={field.name}
                        type={showPassword ? "text" : "password"}
                        autoComplete="current-password"
                        placeholder="Enter your master password"
                        value={field.state.value}
                        onBlur={field.handleBlur}
                        onChange={(event) =>
                          field.handleChange(event.target.value)
                        }
                        aria-invalid={Boolean(errorMessage)}
                        aria-describedby={
                          errorMessage ? "master-password-error" : undefined
                        }
                      />
                      <InputGroupAddon align="inline-end">
                        <InputGroupButton
                          type="button"
                          aria-label={
                            showPassword ? "Hide password" : "Show password"
                          }
                          onClick={() => setShowPassword((value) => !value)}
                        >
                          <HugeiconsIcon
                            icon={showPassword ? ViewOffIcon : EyeIcon}
                            size={16}
                          />
                        </InputGroupButton>
                      </InputGroupAddon>
                    </InputGroup>
                    {errorMessage ? (
                      <p
                        id="master-password-error"
                        className="text-destructive text-sm"
                        role="alert"
                      >
                        {errorMessage}
                      </p>
                    ) : null}
                  </div>
                );
              }}
            </form.Field>

            <form.Subscribe
              selector={(state) => ({
                isSubmitting: state.isSubmitting,
              })}
            >
              {({ isSubmitting }) => (
                <div className="space-y-2">
                  <Button type="submit" disabled={isSubmitting} className="w-full">
                    Log in with master password
                  </Button>
                  <div className="relative py-1">
                    <Separator />
                    <span className="bg-card text-muted-foreground absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 px-2 text-xs">
                      or
                    </span>
                  </div>
                  <Button
                    type="button"
                    variant="outline"
                    disabled={isSubmitting}
                    onClick={() => onDeviceLogin?.()}
                    className="w-full"
                  >
                    <HugeiconsIcon icon={LaptopCheckIcon} size={16} />
                    Log in with device
                  </Button>
                  <Button
                    type="button"
                    variant="ghost"
                    disabled={isSubmitting}
                    onClick={onBack}
                    className="w-full"
                  >
                    Back
                  </Button>
                </div>
              )}
            </form.Subscribe>
          </form>
        </CardContent>
      </Card>
      <p className="text-muted-foreground text-center text-sm">
        New here?{" "}
        <Button
          variant="link"
          type="button"
          className="h-auto p-0 text-sm"
          onClick={onRegisterClick}
        >
          Register
        </Button>
      </p>
    </div>
  );
}
