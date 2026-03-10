import { isTauri } from "@/platform";
import type { PlayerAdapter } from "@/player";
import { TauriMpvAdapter } from "./tauriMpvAdapter";
import { WebVideoAdapter } from "./webVideoAdapter";

export const createPlayerAdapter = (): PlayerAdapter => {
  if (isTauri()) {
    return new TauriMpvAdapter();
  }

  return new WebVideoAdapter();
};
