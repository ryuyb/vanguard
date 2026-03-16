import { useForm } from "@tanstack/react-form";
import { LoaderCircle, Plus, X } from "lucide-react";
import { useEffect } from "react";
import { useTranslation } from "react-i18next";
import type { SyncCipher, VaultFolderItemDto } from "@/bindings";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Textarea } from "@/components/ui/textarea";

type CipherFormDialogProps = {
  open: boolean;
  mode: "create" | "edit";
  initialCipher?: SyncCipher | null;
  folderId?: string | null;
  folders?: VaultFolderItemDto[];
  onOpenChange: (open: boolean) => void;
  onConfirm: (cipher: SyncCipher) => void;
  isLoading?: boolean;
};

const CIPHER_TYPE_LOGIN = 1;
const CIPHER_TYPE_NOTE = 2;
const CIPHER_TYPE_CARD = 3;
const CIPHER_TYPE_SSH_KEY = 7;

const CUSTOM_FIELD_TYPE_TEXT = 0;
const CUSTOM_FIELD_TYPE_HIDDEN = 1;
const CUSTOM_FIELD_TYPE_BOOLEAN = 2;
const CUSTOM_FIELD_TYPE_LINKED = 3;

export function CipherFormDialog({
  open,
  mode,
  initialCipher,
  folderId,
  folders = [],
  onOpenChange,
  onConfirm,
  isLoading = false,
}: CipherFormDialogProps) {
  const { t } = useTranslation();
  const form = useForm({
    defaultValues: {
      cipherType: CIPHER_TYPE_LOGIN,
      name: "",
      folderId: "",
      username: "",
      password: "",
      totp: "",
      uris: [""],
      notes: "",
      customFields: [] as Array<{ name: string; value: string; type: number }>,
      cardholderName: "",
      cardNumber: "",
      cardBrand: "",
      expMonth: "",
      expYear: "",
      securityCode: "",
      sshPrivateKey: "",
      sshPublicKey: "",
      sshFingerprint: "",
    },
    onSubmit: ({ value }) => {
      const validUris = value.uris
        .filter((u) => u.trim())
        .map((u) => ({ uri: u || null, match: null, uri_checksum: null }));
      const firstUri = validUris.length > 0 ? validUris[0].uri : null;

      const cipher: SyncCipher = {
        id: mode === "edit" && initialCipher ? initialCipher.id : "",
        organization_id: initialCipher?.organization_id ?? null,
        folder_id: value.folderId || null,
        type: value.cipherType,
        name: value.name || null,
        notes: value.notes || null,
        favorite: initialCipher?.favorite ?? false,
        fields: value.customFields.map((f) => ({
          name: f.name || null,
          value: f.value || null,
          type: f.type,
          linked_id: null,
        })),
        login:
          value.cipherType === CIPHER_TYPE_LOGIN
            ? {
                uri: firstUri,
                uris: validUris,
                username: value.username || null,
                password: value.password || null,
                password_revision_date: null,
                totp: value.totp || null,
                autofill_on_page_load: null,
                fido2_credentials: [],
              }
            : null,
        data:
          value.cipherType === CIPHER_TYPE_LOGIN
            ? {
                name: value.name || null,
                notes: value.notes || null,
                fields: value.customFields.map((f) => ({
                  name: f.name || null,
                  value: f.value || null,
                  type: f.type,
                  linked_id: null,
                })),
                password_history: [],
                uri: firstUri,
                uris: validUris,
                username: value.username || null,
                password: value.password || null,
                password_revision_date: null,
                totp: value.totp || null,
                autofill_on_page_load: null,
                fido2_credentials: [],
                type: null,
                cardholder_name: null,
                brand: null,
                number: null,
                exp_month: null,
                exp_year: null,
                code: null,
                title: null,
                first_name: null,
                middle_name: null,
                last_name: null,
                address1: null,
                address2: null,
                address3: null,
                city: null,
                state: null,
                postal_code: null,
                country: null,
                company: null,
                email: null,
                phone: null,
                ssn: null,
                passport_number: null,
                license_number: null,
                private_key: null,
                public_key: null,
                key_fingerprint: null,
              }
            : value.cipherType === CIPHER_TYPE_CARD
              ? {
                  name: value.name || null,
                  notes: value.notes || null,
                  fields: value.customFields.map((f) => ({
                    name: f.name || null,
                    value: f.value || null,
                    type: f.type,
                    linked_id: null,
                  })),
                  password_history: [],
                  uri: null,
                  uris: [],
                  username: null,
                  password: null,
                  password_revision_date: null,
                  totp: null,
                  autofill_on_page_load: null,
                  fido2_credentials: [],
                  type: null,
                  cardholder_name: value.cardholderName || null,
                  brand: value.cardBrand || null,
                  number: value.cardNumber || null,
                  exp_month: value.expMonth || null,
                  exp_year: value.expYear || null,
                  code: value.securityCode || null,
                  title: null,
                  first_name: null,
                  middle_name: null,
                  last_name: null,
                  address1: null,
                  address2: null,
                  address3: null,
                  city: null,
                  state: null,
                  postal_code: null,
                  country: null,
                  company: null,
                  email: null,
                  phone: null,
                  ssn: null,
                  passport_number: null,
                  license_number: null,
                  private_key: null,
                  public_key: null,
                  key_fingerprint: null,
                }
              : null,
        secure_note: value.cipherType === CIPHER_TYPE_NOTE ? { type: 0 } : null,
        card:
          value.cipherType === CIPHER_TYPE_CARD
            ? {
                cardholder_name: value.cardholderName || null,
                brand: value.cardBrand || null,
                number: value.cardNumber || null,
                exp_month: value.expMonth || null,
                exp_year: value.expYear || null,
                code: value.securityCode || null,
              }
            : null,
        identity: null,
        ssh_key:
          value.cipherType === CIPHER_TYPE_SSH_KEY
            ? {
                private_key: value.sshPrivateKey || null,
                public_key: value.sshPublicKey || null,
                key_fingerprint: value.sshFingerprint || null,
              }
            : null,
        password_history: [],
        attachments: [],
        collection_ids: [],
        creation_date: initialCipher?.creation_date ?? null,
        deleted_date: null,
        revision_date: initialCipher?.revision_date ?? null,
        archived_date: null,
        key: null,
        edit: true,
        view_password: true,
        organization_use_totp: false,
        reprompt: null,
        permissions: null,
        object: null,
      };

      onConfirm(cipher);
    },
  });

  useEffect(() => {
    if (open) {
      if (initialCipher) {
        const existingUris =
          initialCipher.login?.uris ?? initialCipher.data?.uris ?? [];
        form.reset({
          cipherType: initialCipher.type ?? CIPHER_TYPE_LOGIN,
          name: initialCipher.name ?? "",
          folderId: initialCipher.folder_id ?? "",
          username:
            initialCipher.login?.username ?? initialCipher.data?.username ?? "",
          password:
            initialCipher.login?.password ?? initialCipher.data?.password ?? "",
          totp: initialCipher.login?.totp ?? initialCipher.data?.totp ?? "",
          uris:
            existingUris.length > 0
              ? existingUris.map((u) => u.uri ?? "").filter(Boolean)
              : [""],
          notes: initialCipher.notes ?? initialCipher.data?.notes ?? "",
          customFields: (initialCipher.fields ?? []).map((f) => ({
            name: f.name ?? "",
            value: f.value ?? "",
            type: f.type ?? 0,
          })),
          cardholderName:
            initialCipher.card?.cardholder_name ??
            initialCipher.data?.cardholder_name ??
            "",
          cardNumber:
            initialCipher.card?.number ?? initialCipher.data?.number ?? "",
          cardBrand:
            initialCipher.card?.brand ?? initialCipher.data?.brand ?? "",
          expMonth:
            initialCipher.card?.exp_month ??
            initialCipher.data?.exp_month ??
            "",
          expYear:
            initialCipher.card?.exp_year ?? initialCipher.data?.exp_year ?? "",
          securityCode:
            initialCipher.card?.code ?? initialCipher.data?.code ?? "",
          sshPrivateKey: initialCipher.ssh_key?.private_key ?? "",
          sshPublicKey: initialCipher.ssh_key?.public_key ?? "",
          sshFingerprint: initialCipher.ssh_key?.key_fingerprint ?? "",
        });
      } else {
        form.reset({
          cipherType: CIPHER_TYPE_LOGIN,
          name: "",
          folderId: folderId ?? "",
          username: "",
          password: "",
          totp: "",
          uris: [""],
          notes: "",
          customFields: [],
          cardholderName: "",
          cardNumber: "",
          cardBrand: "",
          expMonth: "",
          expYear: "",
          securityCode: "",
          sshPrivateKey: "",
          sshPublicKey: "",
          sshFingerprint: "",
        });
      }
    }
  }, [open, initialCipher, folderId, form]);

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-2xl sm:max-w-2xl max-h-[85vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle className="text-xl font-bold text-slate-900">
            {mode === "create"
              ? t("vault.dialogs.cipherForm.createTitle")
              : t("vault.dialogs.cipherForm.editTitle")}
          </DialogTitle>
          <DialogDescription className="text-sm text-slate-600">
            {mode === "create"
              ? t("vault.dialogs.cipherForm.createDescription")
              : t("vault.dialogs.cipherForm.editDescription")}
          </DialogDescription>
        </DialogHeader>

        <form
          onSubmit={(e) => {
            e.preventDefault();
            form.handleSubmit();
          }}
          className="space-y-5"
        >
          <form.Field name="cipherType">
            {(field) => (
              <div className="space-y-2">
                <Label htmlFor="cipher-type" className="text-sm font-semibold">
                  {t("vault.dialogs.cipherForm.fields.type")}
                </Label>
                <Select
                  value={String(field.state.value)}
                  onValueChange={(value) => field.handleChange(Number(value))}
                  disabled={mode === "edit"}
                >
                  <SelectTrigger id="cipher-type" className="h-10 w-full">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent className="w-full">
                    <SelectItem value={String(CIPHER_TYPE_LOGIN)}>
                      {t("vault.dialogs.cipherForm.types.login")}
                    </SelectItem>
                    <SelectItem value={String(CIPHER_TYPE_NOTE)}>
                      {t("vault.dialogs.cipherForm.types.note")}
                    </SelectItem>
                    <SelectItem value={String(CIPHER_TYPE_CARD)}>
                      {t("vault.dialogs.cipherForm.types.card")}
                    </SelectItem>
                    <SelectItem value={String(CIPHER_TYPE_SSH_KEY)}>
                      {t("vault.dialogs.cipherForm.types.sshKey")}
                    </SelectItem>
                  </SelectContent>
                </Select>
              </div>
            )}
          </form.Field>

          <form.Field
            name="name"
            validators={{
              onChange: ({ value }) =>
                !value.trim()
                  ? t("vault.dialogs.cipherForm.validation.nameRequired")
                  : undefined,
            }}
          >
            {(field) => (
              <div className="space-y-2">
                <Label htmlFor="cipher-name" className="text-sm font-semibold">
                  {t("vault.dialogs.cipherForm.fields.name")}{" "}
                  <span className="text-red-500">*</span>
                </Label>
                <Input
                  id="cipher-name"
                  value={field.state.value}
                  onChange={(e) => field.handleChange(e.target.value)}
                  placeholder={t("vault.dialogs.cipherForm.placeholders.name")}
                  required
                  className="h-10"
                />
              </div>
            )}
          </form.Field>

          <form.Field name="folderId">
            {(field) => (
              <div className="space-y-2">
                <Label
                  htmlFor="cipher-folder"
                  className="text-sm font-semibold"
                >
                  {t("vault.dialogs.cipherForm.fields.folder")}
                </Label>
                <Select
                  value={field.state.value || "no-folder"}
                  onValueChange={(value) =>
                    field.handleChange(value === "no-folder" ? "" : value)
                  }
                >
                  <SelectTrigger id="cipher-folder" className="h-10 w-full">
                    <SelectValue
                      placeholder={t("vault.dialogs.cipherForm.noFolder")}
                    />
                  </SelectTrigger>
                  <SelectContent className="w-full max-h-60">
                    <SelectItem value="no-folder">
                      {t("vault.dialogs.cipherForm.noFolder")}
                    </SelectItem>
                    {folders.map((folder) => (
                      <SelectItem key={folder.id} value={folder.id}>
                        {folder.name || t("vault.page.folders.untitledFolder")}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
            )}
          </form.Field>

          <form.Subscribe selector={(state) => state.values.cipherType}>
            {(cipherType) =>
              cipherType === CIPHER_TYPE_LOGIN && (
                <>
                  <form.Field name="username">
                    {(field) => (
                      <div className="space-y-2">
                        <Label
                          htmlFor="cipher-username"
                          className="text-sm font-semibold"
                        >
                          {t("vault.dialogs.cipherForm.fields.username")}
                        </Label>
                        <Input
                          id="cipher-username"
                          value={field.state.value}
                          onChange={(e) => field.handleChange(e.target.value)}
                          placeholder={t(
                            "vault.dialogs.cipherForm.placeholders.username",
                          )}
                          className="h-10"
                        />
                      </div>
                    )}
                  </form.Field>

                  <form.Field name="password">
                    {(field) => (
                      <div className="space-y-2">
                        <Label
                          htmlFor="cipher-password"
                          className="text-sm font-semibold"
                        >
                          {t("vault.dialogs.cipherForm.fields.password")}
                        </Label>
                        <Input
                          id="cipher-password"
                          type="password"
                          value={field.state.value}
                          onChange={(e) => field.handleChange(e.target.value)}
                          placeholder={t(
                            "vault.dialogs.cipherForm.placeholders.password",
                          )}
                          className="h-10"
                        />
                      </div>
                    )}
                  </form.Field>

                  <form.Field name="totp">
                    {(field) => (
                      <div className="space-y-2">
                        <Label
                          htmlFor="cipher-totp"
                          className="text-sm font-semibold"
                        >
                          {t("vault.dialogs.cipherForm.fields.totp")}
                        </Label>
                        <Input
                          id="cipher-totp"
                          value={field.state.value}
                          onChange={(e) => field.handleChange(e.target.value)}
                          placeholder={t(
                            "vault.dialogs.cipherForm.placeholders.totp",
                          )}
                          className="h-10"
                        />
                      </div>
                    )}
                  </form.Field>

                  <form.Field name="uris">
                    {(field) => (
                      <div className="space-y-3">
                        <Label className="text-sm font-semibold">
                          {t("vault.dialogs.cipherForm.fields.uris")}
                        </Label>
                        {field.state.value.map((uri, index) => (
                          <div
                            key={`uri-${index}`}
                            className="flex gap-2 items-center"
                          >
                            <Input
                              value={uri}
                              onChange={(e) => {
                                const updated = [...field.state.value];
                                updated[index] = e.target.value;
                                field.handleChange(updated);
                              }}
                              placeholder={t(
                                "vault.dialogs.cipherForm.placeholders.uri",
                              )}
                              type="url"
                              className="h-10 flex-1"
                            />
                            {field.state.value.length > 1 && (
                              <Button
                                type="button"
                                variant="ghost"
                                size="sm"
                                onClick={() => {
                                  field.handleChange(
                                    field.state.value.filter(
                                      (_, i) => i !== index,
                                    ),
                                  );
                                }}
                                className="h-10 px-3 shrink-0"
                              >
                                <X className="size-4" />
                              </Button>
                            )}
                          </div>
                        ))}
                        <Button
                          type="button"
                          variant="outline"
                          onClick={() =>
                            field.handleChange([...field.state.value, ""])
                          }
                          className="w-full h-10"
                        >
                          <Plus className="size-4" />
                          {t("vault.dialogs.cipherForm.actions.addUri")}
                        </Button>
                      </div>
                    )}
                  </form.Field>
                </>
              )
            }
          </form.Subscribe>

          <form.Subscribe selector={(state) => state.values.cipherType}>
            {(cipherType) =>
              cipherType === CIPHER_TYPE_CARD && (
                <>
                  <form.Field name="cardholderName">
                    {(field) => (
                      <div className="space-y-2">
                        <Label
                          htmlFor="cipher-cardholder-name"
                          className="text-sm font-semibold"
                        >
                          {t("vault.dialogs.cipherForm.fields.cardholderName")}
                        </Label>
                        <Input
                          id="cipher-cardholder-name"
                          value={field.state.value}
                          onChange={(e) => field.handleChange(e.target.value)}
                          placeholder={t(
                            "vault.dialogs.cipherForm.placeholders.cardholderName",
                          )}
                          className="h-10"
                        />
                      </div>
                    )}
                  </form.Field>

                  <form.Field name="cardNumber">
                    {(field) => (
                      <div className="space-y-2">
                        <Label
                          htmlFor="cipher-card-number"
                          className="text-sm font-semibold"
                        >
                          {t("vault.dialogs.cipherForm.fields.cardNumber")}
                        </Label>
                        <Input
                          id="cipher-card-number"
                          value={field.state.value}
                          onChange={(e) => field.handleChange(e.target.value)}
                          placeholder={t(
                            "vault.dialogs.cipherForm.placeholders.cardNumber",
                          )}
                          className="h-10"
                        />
                      </div>
                    )}
                  </form.Field>

                  <form.Field name="cardBrand">
                    {(field) => (
                      <div className="space-y-2">
                        <Label
                          htmlFor="cipher-card-brand"
                          className="text-sm font-semibold"
                        >
                          {t("vault.dialogs.cipherForm.fields.cardBrand")}
                        </Label>
                        <Select
                          value={field.state.value}
                          onValueChange={field.handleChange}
                        >
                          <SelectTrigger
                            id="cipher-card-brand"
                            className="h-10 w-full"
                          >
                            <SelectValue
                              placeholder={t(
                                "vault.dialogs.cipherForm.placeholders.cardBrand",
                              )}
                            />
                          </SelectTrigger>
                          <SelectContent>
                            <SelectItem value="Visa">
                              {t("vault.dialogs.cipherForm.brands.visa")}
                            </SelectItem>
                            <SelectItem value="Mastercard">
                              {t("vault.dialogs.cipherForm.brands.mastercard")}
                            </SelectItem>
                            <SelectItem value="American Express">
                              {t(
                                "vault.dialogs.cipherForm.brands.americanExpress",
                              )}
                            </SelectItem>
                            <SelectItem value="Discover">
                              {t("vault.dialogs.cipherForm.brands.discover")}
                            </SelectItem>
                            <SelectItem value="UnionPay">
                              {t("vault.dialogs.cipherForm.brands.unionPay")}
                            </SelectItem>
                            <SelectItem value="JCB">
                              {t("vault.dialogs.cipherForm.brands.jcb")}
                            </SelectItem>
                            <SelectItem value="Other">
                              {t("vault.dialogs.cipherForm.brands.other")}
                            </SelectItem>
                          </SelectContent>
                        </Select>
                      </div>
                    )}
                  </form.Field>

                  <div className="grid grid-cols-2 gap-4">
                    <form.Field name="expMonth">
                      {(field) => (
                        <div className="space-y-2">
                          <Label
                            htmlFor="cipher-exp-month"
                            className="text-sm font-semibold"
                          >
                            {t("vault.dialogs.cipherForm.fields.expMonth")}
                          </Label>
                          <Select
                            value={field.state.value}
                            onValueChange={field.handleChange}
                          >
                            <SelectTrigger
                              id="cipher-exp-month"
                              className="h-10 w-full"
                            >
                              <SelectValue
                                placeholder={t(
                                  "vault.dialogs.cipherForm.placeholders.month",
                                )}
                              />
                            </SelectTrigger>
                            <SelectContent>
                              {Array.from({ length: 12 }, (_, i) => {
                                const month = String(i + 1).padStart(2, "0");
                                return (
                                  <SelectItem key={month} value={month}>
                                    {month}
                                  </SelectItem>
                                );
                              })}
                            </SelectContent>
                          </Select>
                        </div>
                      )}
                    </form.Field>

                    <form.Field name="expYear">
                      {(field) => (
                        <div className="space-y-2">
                          <Label
                            htmlFor="cipher-exp-year"
                            className="text-sm font-semibold"
                          >
                            {t("vault.dialogs.cipherForm.fields.expYear")}
                          </Label>
                          <Select
                            value={field.state.value}
                            onValueChange={field.handleChange}
                          >
                            <SelectTrigger
                              id="cipher-exp-year"
                              className="h-10 w-full"
                            >
                              <SelectValue
                                placeholder={t(
                                  "vault.dialogs.cipherForm.placeholders.year",
                                )}
                              />
                            </SelectTrigger>
                            <SelectContent>
                              {Array.from({ length: 20 }, (_, i) => {
                                const year = String(
                                  new Date().getFullYear() + i,
                                );
                                return (
                                  <SelectItem key={year} value={year}>
                                    {year}
                                  </SelectItem>
                                );
                              })}
                            </SelectContent>
                          </Select>
                        </div>
                      )}
                    </form.Field>
                  </div>

                  <form.Field name="securityCode">
                    {(field) => (
                      <div className="space-y-2">
                        <Label
                          htmlFor="cipher-security-code"
                          className="text-sm font-semibold"
                        >
                          {t("vault.dialogs.cipherForm.fields.securityCode")}
                        </Label>
                        <Input
                          id="cipher-security-code"
                          type="password"
                          value={field.state.value}
                          onChange={(e) => field.handleChange(e.target.value)}
                          placeholder={t(
                            "vault.dialogs.cipherForm.placeholders.securityCode",
                          )}
                          maxLength={4}
                          className="h-10"
                        />
                      </div>
                    )}
                  </form.Field>
                </>
              )
            }
          </form.Subscribe>

          <form.Subscribe selector={(state) => state.values.cipherType}>
            {(cipherType) =>
              cipherType === CIPHER_TYPE_SSH_KEY && (
                <>
                  <form.Field name="sshPrivateKey">
                    {(field) => (
                      <div className="space-y-2">
                        <Label
                          htmlFor="cipher-ssh-private-key"
                          className="text-sm font-semibold"
                        >
                          {t("vault.dialogs.cipherForm.fields.sshPrivateKey")}
                        </Label>
                        <Textarea
                          id="cipher-ssh-private-key"
                          value={field.state.value}
                          onChange={(e) => field.handleChange(e.target.value)}
                          placeholder={t(
                            "vault.dialogs.cipherForm.placeholders.sshPrivateKey",
                          )}
                          rows={6}
                          className="resize-none font-mono text-xs break-all w-full"
                        />
                      </div>
                    )}
                  </form.Field>

                  <form.Field name="sshPublicKey">
                    {(field) => (
                      <div className="space-y-2">
                        <Label
                          htmlFor="cipher-ssh-public-key"
                          className="text-sm font-semibold"
                        >
                          {t("vault.dialogs.cipherForm.fields.sshPublicKey")}
                        </Label>
                        <Textarea
                          id="cipher-ssh-public-key"
                          value={field.state.value}
                          onChange={(e) => field.handleChange(e.target.value)}
                          placeholder={t(
                            "vault.dialogs.cipherForm.placeholders.sshPublicKey",
                          )}
                          rows={3}
                          className="resize-none font-mono text-xs break-all w-full"
                        />
                      </div>
                    )}
                  </form.Field>

                  <form.Field name="sshFingerprint">
                    {(field) => (
                      <div className="space-y-2">
                        <Label
                          htmlFor="cipher-ssh-fingerprint"
                          className="text-sm font-semibold"
                        >
                          {t("vault.dialogs.cipherForm.fields.sshFingerprint")}
                        </Label>
                        <Input
                          id="cipher-ssh-fingerprint"
                          value={field.state.value}
                          onChange={(e) => field.handleChange(e.target.value)}
                          placeholder={t(
                            "vault.dialogs.cipherForm.placeholders.sshFingerprint",
                          )}
                          className="h-10 font-mono text-xs break-all w-full"
                        />
                      </div>
                    )}
                  </form.Field>
                </>
              )
            }
          </form.Subscribe>

          <form.Field name="notes">
            {(field) => (
              <div className="space-y-2">
                <Label htmlFor="cipher-notes" className="text-sm font-semibold">
                  {t("vault.dialogs.cipherForm.fields.notes")}
                </Label>
                <Textarea
                  id="cipher-notes"
                  value={field.state.value}
                  onChange={(e) => field.handleChange(e.target.value)}
                  placeholder={t("vault.dialogs.cipherForm.placeholders.notes")}
                  rows={4}
                  className="resize-none"
                />
              </div>
            )}
          </form.Field>

          <form.Field name="customFields">
            {(field) => (
              <>
                {field.state.value.length > 0 && (
                  <div className="space-y-3">
                    <Label className="text-sm font-semibold">
                      {t("vault.dialogs.cipherForm.fields.customFields")}
                    </Label>
                    {field.state.value.map((customField, index) => (
                      <div
                        key={`custom-field-${index}`}
                        className="space-y-2 rounded-lg border border-slate-200 bg-slate-50 p-3"
                      >
                        <div className="flex gap-2 items-start">
                          <Input
                            value={customField.name}
                            onChange={(e) => {
                              const updated = [...field.state.value];
                              updated[index] = {
                                ...updated[index],
                                name: e.target.value,
                              };
                              field.handleChange(updated);
                            }}
                            placeholder={t(
                              "vault.dialogs.cipherForm.placeholders.customFieldName",
                            )}
                            className="h-10 flex-1"
                          />
                          <Select
                            value={String(customField.type)}
                            onValueChange={(value) => {
                              const updated = [...field.state.value];
                              updated[index] = {
                                ...updated[index],
                                type: Number(value),
                              };
                              field.handleChange(updated);
                            }}
                          >
                            <SelectTrigger className="h-10 w-32">
                              <SelectValue />
                            </SelectTrigger>
                            <SelectContent>
                              <SelectItem
                                value={String(CUSTOM_FIELD_TYPE_TEXT)}
                              >
                                {t(
                                  "vault.dialogs.cipherForm.customFieldTypes.text",
                                )}
                              </SelectItem>
                              <SelectItem
                                value={String(CUSTOM_FIELD_TYPE_HIDDEN)}
                              >
                                {t(
                                  "vault.dialogs.cipherForm.customFieldTypes.hidden",
                                )}
                              </SelectItem>
                              <SelectItem
                                value={String(CUSTOM_FIELD_TYPE_BOOLEAN)}
                              >
                                {t(
                                  "vault.dialogs.cipherForm.customFieldTypes.boolean",
                                )}
                              </SelectItem>
                              <SelectItem
                                value={String(CUSTOM_FIELD_TYPE_LINKED)}
                              >
                                {t(
                                  "vault.dialogs.cipherForm.customFieldTypes.linked",
                                )}
                              </SelectItem>
                            </SelectContent>
                          </Select>
                          <Button
                            type="button"
                            variant="ghost"
                            size="sm"
                            onClick={() => {
                              field.handleChange(
                                field.state.value.filter((_, i) => i !== index),
                              );
                            }}
                            className="h-10 px-3 shrink-0"
                          >
                            <X className="size-4" />
                          </Button>
                        </div>
                        {customField.type === CUSTOM_FIELD_TYPE_BOOLEAN ? (
                          <Select
                            value={customField.value}
                            onValueChange={(value) => {
                              const updated = [...field.state.value];
                              updated[index] = {
                                ...updated[index],
                                value,
                              };
                              field.handleChange(updated);
                            }}
                          >
                            <SelectTrigger className="h-10 w-full">
                              <SelectValue
                                placeholder={t(
                                  "vault.dialogs.cipherForm.placeholders.booleanValue",
                                )}
                              />
                            </SelectTrigger>
                            <SelectContent>
                              <SelectItem value="true">
                                {t("vault.detail.boolean.true")}
                              </SelectItem>
                              <SelectItem value="false">
                                {t("vault.detail.boolean.false")}
                              </SelectItem>
                            </SelectContent>
                          </Select>
                        ) : (
                          <Input
                            value={customField.value}
                            onChange={(e) => {
                              const updated = [...field.state.value];
                              updated[index] = {
                                ...updated[index],
                                value: e.target.value,
                              };
                              field.handleChange(updated);
                            }}
                            placeholder={
                              customField.type === CUSTOM_FIELD_TYPE_HIDDEN
                                ? t(
                                    "vault.dialogs.cipherForm.placeholders.hiddenValue",
                                  )
                                : customField.type === CUSTOM_FIELD_TYPE_LINKED
                                  ? t(
                                      "vault.dialogs.cipherForm.placeholders.linkValue",
                                    )
                                  : t(
                                      "vault.dialogs.cipherForm.placeholders.customFieldValue",
                                    )
                            }
                            type={
                              customField.type === CUSTOM_FIELD_TYPE_HIDDEN
                                ? "password"
                                : "text"
                            }
                            className="h-10"
                          />
                        )}
                      </div>
                    ))}
                  </div>
                )}

                <Button
                  type="button"
                  variant="outline"
                  onClick={() =>
                    field.handleChange([
                      ...field.state.value,
                      { name: "", value: "", type: CUSTOM_FIELD_TYPE_TEXT },
                    ])
                  }
                  className="w-full h-10"
                >
                  <Plus className="size-4" />
                  {t("vault.dialogs.cipherForm.actions.addCustomField")}
                </Button>
              </>
            )}
          </form.Field>

          <DialogFooter className="gap-2">
            <Button
              type="button"
              variant="outline"
              onClick={() => onOpenChange(false)}
              disabled={isLoading}
              className="h-10"
            >
              {t("common.actions.cancel")}
            </Button>
            <form.Subscribe
              selector={(state) =>
                [state.values.name, state.canSubmit] as const
              }
            >
              {([name, canSubmit]) => (
                <Button
                  type="submit"
                  disabled={isLoading || !name?.trim() || !canSubmit}
                  className="h-10 bg-blue-600 hover:bg-blue-700"
                >
                  {isLoading ? (
                    <>
                      <LoaderCircle className="size-4 animate-spin" />
                      {mode === "create"
                        ? t("vault.dialogs.cipherForm.actions.creating")
                        : t("vault.dialogs.cipherForm.actions.saving")}
                    </>
                  ) : mode === "create" ? (
                    t("vault.page.actions.create")
                  ) : (
                    t("common.actions.save")
                  )}
                </Button>
              )}
            </form.Subscribe>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}
