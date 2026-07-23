<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, useId, watch } from "vue";

export interface P3SelectOption {
  value: string;
  label: string;
}

const props = defineProps<{
  modelValue: string;
  options: P3SelectOption[];
  label: string;
}>();

const emit = defineEmits<{
  "update:modelValue": [value: string];
}>();

const root = ref<HTMLElement | null>(null);
const trigger = ref<HTMLButtonElement | null>(null);
const open = ref(false);
const activeIndex = ref(0);
const componentId = useId();
const listboxId = `${componentId}-listbox`;
const selectedIndex = computed(() =>
  Math.max(
    0,
    props.options.findIndex((option) => option.value === props.modelValue),
  ),
);
const selectedLabel = computed(
  () => props.options[selectedIndex.value]?.label ?? "请选择",
);

watch(selectedIndex, (index) => {
  if (!open.value) activeIndex.value = index;
});

function optionId(index: number) {
  return `${componentId}-option-${index}`;
}

function openMenu() {
  activeIndex.value = selectedIndex.value;
  open.value = true;
}

function closeMenu() {
  open.value = false;
}

function toggleMenu() {
  if (open.value) closeMenu();
  else openMenu();
}

function moveActive(offset: number) {
  if (!open.value) openMenu();
  const length = props.options.length;
  activeIndex.value = (activeIndex.value + offset + length) % length;
}

function select(index: number) {
  const option = props.options[index];
  if (!option) return;
  emit("update:modelValue", option.value);
  activeIndex.value = index;
  closeMenu();
  trigger.value?.focus();
}

function onKeydown(event: KeyboardEvent) {
  switch (event.key) {
    case "ArrowDown":
      event.preventDefault();
      moveActive(1);
      break;
    case "ArrowUp":
      event.preventDefault();
      moveActive(-1);
      break;
    case "Home":
      if (!open.value) return;
      event.preventDefault();
      activeIndex.value = 0;
      break;
    case "End":
      if (!open.value) return;
      event.preventDefault();
      activeIndex.value = props.options.length - 1;
      break;
    case "Enter":
    case " ":
      event.preventDefault();
      if (open.value) select(activeIndex.value);
      else openMenu();
      break;
    case "Escape":
      if (!open.value) return;
      event.preventDefault();
      closeMenu();
      break;
    case "Tab":
      closeMenu();
      break;
  }
}

function onDocumentPointerDown(event: PointerEvent) {
  if (!root.value?.contains(event.target as Node)) closeMenu();
}

onMounted(() => document.addEventListener("pointerdown", onDocumentPointerDown));
onBeforeUnmount(() =>
  document.removeEventListener("pointerdown", onDocumentPointerDown),
);
</script>

<template>
  <div ref="root" class="p3-select" :class="{ open }">
    <button
      ref="trigger"
      class="p3-select-trigger"
      type="button"
      role="combobox"
      aria-haspopup="listbox"
      :aria-label="label"
      :aria-expanded="open"
      :aria-controls="listboxId"
      :aria-activedescendant="open ? optionId(activeIndex) : undefined"
      @click="toggleMenu"
      @keydown="onKeydown"
    >
      <span>{{ selectedLabel }}</span>
      <i class="ph-thin ph-caret-down" aria-hidden="true"></i>
    </button>

    <Transition name="p3-select-menu">
      <ul v-if="open" :id="listboxId" class="p3-select-options" role="listbox">
        <li
          v-for="(option, index) in options"
          :id="optionId(index)"
          :key="option.value"
          role="option"
          :aria-selected="option.value === modelValue"
          :class="{
            active: index === activeIndex,
            selected: option.value === modelValue,
          }"
          @mouseenter="activeIndex = index"
          @mousedown.prevent
          @click="select(index)"
        >
          <span>{{ option.label }}</span>
          <i
            v-if="option.value === modelValue"
            class="ph-thin ph-check"
            aria-hidden="true"
          ></i>
        </li>
      </ul>
    </Transition>
  </div>
</template>
