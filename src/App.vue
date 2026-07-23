<script setup lang="ts">
import { computed, onMounted } from "vue";
import { useRoute, useRouter } from "vue-router";
import AppShell from "@/components/common/AppShell.vue";
import StatePanel from "@/components/common/StatePanel.vue";
import WindowControls from "@/components/common/WindowControls.vue";
import { useAppStore } from "@/stores/app";
import { useProfileStore } from "@/stores/profile";

const appStore = useAppStore();
const profileStore = useProfileStore();
const route = useRoute();
const router = useRouter();

const standalone = computed(() => Boolean(route.meta.standalone));

onMounted(async () => {
  await appStore.bootstrap();
  profileStore.profile = appStore.state?.profile ?? null;

  if (!appStore.onboardingComplete && route.name !== "onboarding") {
    await router.replace({ name: "onboarding" });
  } else if (appStore.onboardingComplete && route.name === "onboarding") {
    await router.replace({ name: "today" });
  }
});
</script>

<template>
  <main class="app-root" aria-label="星迹桌面应用">
    <WindowControls />
    <StatePanel
      v-if="!appStore.ready"
      icon="ph-stars"
      title="正在辨认你的星光"
      message="请稍候，宇宙简报正在展开。"
      loading
    />
    <StatePanel
      v-else-if="appStore.error"
      icon="ph-cloud-slash"
      title="暂时没有连接上"
      :message="appStore.error.message"
      action-label="重新尝试"
      @action="appStore.bootstrap"
    />
    <RouterView v-else-if="standalone" />
    <AppShell v-else>
      <RouterView />
    </AppShell>
  </main>
</template>
