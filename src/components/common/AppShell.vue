<script setup lang="ts">
import SideNavigation from "@/components/common/SideNavigation.vue";
</script>

<template>
  <div class="app-shell">
    <SideNavigation />
    <section class="app-content">
      <slot />
    </section>
  </div>
</template>
