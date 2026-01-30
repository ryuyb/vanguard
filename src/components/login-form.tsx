import { useForm } from "@tanstack/react-form";
import { z } from "zod";

import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Checkbox } from "@/components/ui/checkbox";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Separator } from "@/components/ui/separator";
import { cn } from "@/lib/utils";

type LoginFormValues = {
  email: string;
  rememberEmail: boolean;
};

type LoginFormProps = {
  className?: string;
  onContinue?: (values: LoginFormValues) => void | Promise<void>;
  onSso?: (values: LoginFormValues) => void | Promise<void>;
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

export function LoginForm({ className, onContinue, onSso }: LoginFormProps) {
  const defaultValues: LoginFormValues = {
    email: "",
    rememberEmail: false,
  };

  const form = useForm({
    defaultValues,
    onSubmit: async ({ value }) => {
      await onContinue?.(value);
    },
  });

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

  const isEmailValid = (value: string) => validateEmail(value) === undefined;

  return (
    <Card className={cn("w-full max-w-md", className)}>
      <CardContent>
        <form
          className="space-y-4"
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
                    onChange={(event) => field.handleChange(event.target.value)}
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
                <Label htmlFor="rememberEmail">Remember this email</Label>
              </div>
            )}
          </form.Field>

          <form.Subscribe
            selector={(state) => ({
              canSubmit: state.canSubmit,
              isSubmitting: state.isSubmitting,
              email: state.values.email,
              rememberEmail: state.values.rememberEmail,
            })}
          >
            {({ canSubmit, isSubmitting, email, rememberEmail }) => {
              const canSso = isEmailValid(email) && !isSubmitting;

              return (
                <div className="space-y-2">
                  <Button
                    type="submit"
                    disabled={!canSubmit || isSubmitting}
                    className="w-full"
                  >
                    Continue to next step
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
                    disabled={!canSso}
                    onClick={() => onSso?.({ email, rememberEmail })}
                    className="w-full"
                  >
                    Use SSO to login
                  </Button>
                </div>
              );
            }}
          </form.Subscribe>
        </form>
      </CardContent>
    </Card>
  );
}
