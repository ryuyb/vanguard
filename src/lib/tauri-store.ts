import { Store } from "@tauri-apps/plugin-store";

type StoreKey<TSchema> = Extract<keyof TSchema, string>;

export const createTauriStore = <TSchema extends object>(
  path: string,
) => {
  const storePromise = Store.load(path);

  return {
    async get<K extends StoreKey<TSchema>>(key: K): Promise<TSchema[K] | null> {
      const store = await storePromise;
      return (await store.get(key)) as TSchema[K] | null;
    },

    async set<K extends StoreKey<TSchema>>(key: K, value: TSchema[K]) {
      const store = await storePromise;
      await store.set(key, value);
      await store.save();
    },

    async delete<K extends StoreKey<TSchema>>(key: K) {
      const store = await storePromise;
      await store.delete(key);
      await store.save();
    },
  };
};

// Define your key/value types here to get type-safe accessors.
export interface SelfHosted {
  serverUrl: string;
  webVaultUrl?: string;
  apiUrl?: string;
  identityUrl?: string;
  notificationsUrl?: string;
  iconsUrl?: string;
}

export interface AppStoreSchema extends Record<string, unknown> {
  serverHost: "bitwarden.com"|"bitwarden.eu"|"self-hosted";
  selfHosted?: SelfHosted;
  email?: string;
}

export const appStore = createTauriStore<AppStoreSchema>("app.store.json");
