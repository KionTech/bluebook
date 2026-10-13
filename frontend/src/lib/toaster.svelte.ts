// The toasts: the flash messages a page arrives with (success, error,
// info), and the errors of a refused form, each shown for four seconds or
// until dismissed, like Laravel's Toaster.vue.
import type { Flash } from "#lib/types.ts";

export type ToastType = "success" | "error" | "info";
export type Toast = { id: number; type: ToastType; message: string };

export const DURATION_MS = 4000;

/** The messages of a flash, in Toaster.vue's order. */
export function flashMessages(
  flash: Flash | null | undefined,
): [ToastType, string][] {
  if (!flash) {
    return [];
  }
  return (["success", "error", "info"] as const)
    .filter((type) => flash[type])
    .map((type) => [type, flash[type] as string]);
}

export class Toaster {
  items: Toast[] = $state([]);
  #counter = 0;

  add(type: ToastType, message: string): void {
    const id = ++this.#counter;
    this.items.push({ id, type, message });
    setTimeout(() => this.dismiss(id), DURATION_MS);
  }

  showFlash(flash: Flash | null | undefined): void {
    for (const [type, message] of flashMessages(flash)) {
      this.add(type, message);
    }
  }

  dismiss(id: number): void {
    this.items = this.items.filter((toast) => toast.id !== id);
  }
}

/** The page's toaster. */
export const toaster = new Toaster();
