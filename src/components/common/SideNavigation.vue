<script setup lang="ts">
import { computed } from "vue";
import { useRoute } from "vue-router";

const route = useRoute();
const items = [
  { name: "today", label: "今日", icon: "ph-star-four", testId: "nav-today" },
  { name: "trail", label: "星迹", icon: "ph-chart-polar", testId: "nav-trail" },
  { name: "explore", label: "探索", icon: "ph-compass", testId: "nav-explore" },
  { name: "profile", label: "我的", icon: "ph-user-circle", testId: "nav-profile" },
] as const;

const activeName = computed(() => String(route.name ?? ""));
</script>

<template>
  <nav class="side-nav" aria-label="主导航">
    <RouterLink class="brand" :to="{ name: 'today' }" aria-label="星迹首页">
      星迹<span aria-hidden="true">·</span>
    </RouterLink>

    <div class="nav-items">
      <RouterLink
        v-for="item in items"
        :key="item.name"
        class="nav-item"
        :class="{ active: activeName === item.name }"
        :to="{ name: item.name }"
        :data-testid="item.testId"
        :aria-current="activeName === item.name ? 'page' : undefined"
      >
        <i class="ph-thin" :class="item.icon" aria-hidden="true"></i>
        <span>{{ item.label }}</span>
      </RouterLink>
    </div>

    <div class="nav-horizon" aria-hidden="true">
      <span class="horizon-star"></span>
      <span class="horizon-line"></span>
      <span class="horizon-moon"></span>
    </div>
  </nav>
</template>
