<script setup lang="ts">
defineProps<{
  icon: string;
  title: string;
  message: string;
  loading?: boolean;
  actionLabel?: string;
}>();

defineEmits<{
  action: [];
}>();
</script>

<template>
  <section class="state-panel" role="status">
    <span class="state-icon" :class="{ pulse: loading }">
      <i class="ph-thin" :class="icon" aria-hidden="true"></i>
    </span>
    <h2>{{ title }}</h2>
    <p>{{ message }}</p>
    <button
      v-if="actionLabel"
      class="secondary-button"
      type="button"
      data-testid="state-action"
      @click="$emit('action')"
    >
      {{ actionLabel }}
    </button>
  </section>
</template>
