<script setup lang="ts">
import { getCurrentWindow } from "@tauri-apps/api/window";

const isTauri = () => "__TAURI_INTERNALS__" in window;

async function minimizeWindow() {
  if (isTauri()) await getCurrentWindow().minimize();
}

async function closeWindow() {
  if (isTauri()) await getCurrentWindow().close();
}
</script>

<template>
  <div class="window-chrome" aria-label="窗口控制">
    <div class="window-drag-region" data-tauri-drag-region></div>
    <div class="window-controls">
      <button
        type="button"
        aria-label="最小化"
        data-testid="window-minimize"
        @click="minimizeWindow"
      >
        <i class="ph-thin ph-minus" aria-hidden="true"></i>
      </button>
      <button
        class="window-close"
        type="button"
        aria-label="关闭"
        data-testid="window-close"
        @click="closeWindow"
      >
        <i class="ph-thin ph-x" aria-hidden="true"></i>
      </button>
    </div>
  </div>
</template>
