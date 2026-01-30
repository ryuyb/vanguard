import { useForm } from "@tanstack/react-form";
import { z } from "zod";

import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { cn } from "@/lib/utils";
import registerIllustration from "@/assets/register.svg";

type RegisterFormValues = {
  email: string;
  name: string;
};

type RegisterFormProps = {
  className?: string;
  onContinue?: (values: RegisterFormValues) => void | Promise<void>;
  onLoginClick?: () => void;
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

export function RegisterForm({
  className,
  onContinue,
  onLoginClick,
}: RegisterFormProps) {
  const defaultValues: RegisterFormValues = {
    email: "",
    name: "",
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

  return (
    <div className={cn("w-full max-w-md space-y-8", className)}>
      <img
        src={registerIllustration}
        alt="Register"
        className="mx-auto h-20"
      />
      <h2 className="text-foreground text-center text-2xl font-semibold">
        Create your account
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
            <form.Field name="email" validators={{ onSubmit: ({ value }) => validateEmail(value) }}>
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

            <form.Field name="name">
              {(field) => (
                <div className="space-y-2">
                  <Label htmlFor={field.name}>Name</Label>
                  <Input
                    id={field.name}
                    name={field.name}
                    type="text"
                    autoComplete="name"
                    placeholder="Your name"
                    value={field.state.value}
                    onBlur={field.handleBlur}
                    onChange={(event) => field.handleChange(event.target.value)}
                  />
                </div>
              )}
            </form.Field>

            <Button type="submit" disabled={form.state.isSubmitting} className="w-full">
              Continue
            </Button>
          </form>
        </CardContent>
      </Card>
      <p className="text-muted-foreground text-center text-sm">
        Already have an account?{" "}
        <Button
          variant="link"
          type="button"
          className="h-auto p-0 text-sm"
          onClick={onLoginClick}
        >
          Log in
        </Button>
      </p>
    </div>
  );
}
