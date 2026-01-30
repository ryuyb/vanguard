import { useForm } from "@tanstack/react-form";
import { useEffect } from "react";
import { z } from "zod";

import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Checkbox } from "@/components/ui/checkbox";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Separator } from "@/components/ui/separator";
import { appStore } from "@/lib/tauri-store";
import { cn } from "@/lib/utils";
import loginIllustration from "@/assets/login.svg";

type LoginFormValues = {
  email: string;
  rememberEmail: boolean;
};

type LoginFormProps = {
  className?: string;
  onContinue?: (values: LoginFormValues) => void | Promise<void>;
  onSso?: (values: LoginFormValues) => void | Promise<void>;
  onRegisterClick?: () => void;
};

const emailSchema = z.email({ message: "Enter a valid email address" });

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

export function LoginForm({
  className,
  onContinue,
  onSso,
  onRegisterClick,
}: LoginFormProps) {
  const defaultValues: LoginFormValues = {
    email: "",
    rememberEmail: false,
  };

  const form = useForm({
    defaultValues,
    onSubmit: async ({ value }) => {
      if (value.rememberEmail) {
        await appStore.set("email", value.email);
      } else {
        await appStore.delete("email");
      }
      await onContinue?.(value);
    },
  });

  useEffect(() => {
    let active = true;

    const loadEmail = async () => {
      const email = await appStore.get("email");
      if (!active || !email) {
        return;
      }
      form.setFieldValue("email", email);
      form.setFieldValue("rememberEmail", true);
    };

    void loadEmail();

    return () => {
      active = false;
    };
  }, [form]);

  const validateEmail = (value: string) => {
    if (value.length === 0) {
      return "Email is required";
    }

    const result = emailSchema.safeParse(value);
    if (result.success) {
      return undefined;
    }
    return result.error.issues[0]?.message ?? "Invalid email address";
  };

  return (
    <div className={cn("w-full max-w-md space-y-8", className)}>
      <img
        src={loginIllustration}
        alt="Login"
        className="mx-auto h-20"
      />
      <h2 className="text-foreground text-center text-2xl font-semibold">
        Log in to Bitwarden
      </h2>
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
              name="email"
              validators={{
                onBlur: ({ value }) => validateEmail(value),
                onSubmit: ({ value }) => validateEmail(value),
              }}
            >
              {(field) => {
                const errorMessage = getErrorMessage(field.state.meta.errors);

                return (
                  <div className="space-y-2">
                    <Label htmlFor={field.name}>Email</Label>
                    <Input
                      id={field.name}
                      name={field.name}
                      type="email"
                      autoComplete="email"
                      placeholder="you@example.com"
                      value={field.state.value}
                      onBlur={field.handleBlur}
                      onChange={(event) =>
                        field.handleChange(event.target.value)
                      }
                      aria-invalid={Boolean(errorMessage)}
                      aria-describedby={errorMessage ? "email-error" : undefined}
                    />
                    {errorMessage ? (
                      <p
                        id="email-error"
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

            <form.Field name="rememberEmail">
              {(field) => (
                <div className="flex items-center gap-2">
                  <Checkbox
                    id="rememberEmail"
                    checked={field.state.value}
                    onCheckedChange={(checked) =>
                      field.handleChange(checked === true)
                    }
                  />
                  <Label htmlFor="rememberEmail">Remember email</Label>
                </div>
              )}
            </form.Field>

            <form.Subscribe
              selector={(state) => ({
                isSubmitting: state.isSubmitting,
                email: state.values.email,
                rememberEmail: state.values.rememberEmail,
              })}
            >
              {({ isSubmitting, email, rememberEmail }) => (
                <div className="space-y-2">
                  <Button type="submit" disabled={isSubmitting} className="w-full">
                    Continue
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
                    onClick={async () => {
                      const errors = await form.validateField("email", "submit");
                      if (errors?.length) {
                        return;
                      }
                      if (rememberEmail) {
                        await appStore.set("email", email);
                      } else {
                        await appStore.delete("email");
                      }
                      await onSso?.({ email, rememberEmail });
                    }}
                    className="w-full"
                  >
                    Use single sign-on
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
