<script setup lang="ts">
defineProps<{
  kicker: string;
  icon: string;
  title: string;
  subtitle: string;
}>();
</script>

<template>
  <header class="page-header">
    <p class="page-kicker">
      <i class="ph-thin" :class="icon" aria-hidden="true"></i>
      {{ kicker }}
    </p>
    <h1>{{ title }}</h1>
    <p class="page-subtitle">{{ subtitle }}</p>
  </header>
</template>
