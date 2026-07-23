import { defineStore } from "pinia";
import { useCommandClient } from "@/services/commands";
import { toAppError } from "@/services/errors";
import type { AppErrorShape, Profile, ProfileInput } from "@/types";

export const useProfileStore = defineStore("profile", {
  state: () => ({
    profile: null as Profile | null,
    loading: false,
    saving: false,
    error: null as AppErrorShape | null,
  }),
  actions: {
    async load() {
      this.loading = true;
      this.error = null;
      try {
        this.profile = await useCommandClient().getProfile();
      } catch (error) {
        this.error = toAppError(error);
      } finally {
        this.loading = false;
      }
    },
    async save(input: ProfileInput) {
      this.saving = true;
      this.error = null;
      try {
        this.profile = await useCommandClient().saveProfile(input);
        return this.profile;
      } catch (error) {
        this.error = toAppError(error);
        throw error;
      } finally {
        this.saving = false;
      }
    },
  },
});
