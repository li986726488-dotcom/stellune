<script setup lang="ts">
import {
  computed,
  nextTick,
  onBeforeUnmount,
  onMounted,
  ref,
  useId,
} from "vue";

type TimeGroup = "hour" | "minute";

const props = withDefaults(
  defineProps<{
    modelValue: string;
    label: string;
    placeholder?: string;
    testId?: string;
    placement?: "top" | "bottom";
  }>(),
  {
    placeholder: "未填写",
    testId: undefined,
    placement: "top",
  },
);

const emit = defineEmits<{
  "update:modelValue": [value: string];
}>();

const hours = Array.from({ length: 24 }, (_, index) =>
  String(index).padStart(2, "0"),
);
const minutes = Array.from({ length: 60 }, (_, index) =>
  String(index).padStart(2, "0"),
);
const root = ref<HTMLElement | null>(null);
const trigger = ref<HTMLButtonElement | null>(null);
const open = ref(false);
const draftHour = ref("12");
const draftMinute = ref("00");
const componentId = useId();
const dialogId = `${componentId}-dialog`;
const displayValue = computed(() => props.modelValue || "");

function parseTime(value: string) {
  const match = /^([01]\d|2[0-3]):([0-5]\d)$/.exec(value);
  return match ? { hour: match[1], minute: match[2] } : null;
}

async function scrollSelectionsIntoView() {
  await nextTick();
  for (const [group, value] of [
    ["hour", draftHour.value],
    ["minute", draftMinute.value],
  ] as const) {
    root.value
      ?.querySelector<HTMLButtonElement>(
        `[data-time-group="${group}"][data-value="${value}"]`,
      )
      ?.scrollIntoView({ block: "center" });
  }
}

function openPicker() {
  const parsed = parseTime(props.modelValue);
  draftHour.value = parsed?.hour ?? "12";
  draftMinute.value = parsed?.minute ?? "00";
  open.value = true;
  void scrollSelectionsIntoView();
}

function closePicker() {
  open.value = false;
}

function togglePicker() {
  if (open.value) closePicker();
  else openPicker();
}

function selectValue(group: TimeGroup, value: string) {
  if (group === "hour") draftHour.value = value;
  else draftMinute.value = value;
}

function confirmTime() {
  emit("update:modelValue", `${draftHour.value}:${draftMinute.value}`);
  closePicker();
  trigger.value?.focus();
}

function clearTime() {
  emit("update:modelValue", "");
  closePicker();
  trigger.value?.focus();
}

function focusOption(group: TimeGroup, index: number) {
  const values = group === "hour" ? hours : minutes;
  const wrapped = (index + values.length) % values.length;
  root.value
    ?.querySelector<HTMLButtonElement>(
      `[data-time-group="${group}"][data-value="${values[wrapped]}"]`,
    )
    ?.focus();
}

function onOptionKeydown(
  event: KeyboardEvent,
  group: TimeGroup,
  index: number,
) {
  if (event.key === "ArrowDown" || event.key === "ArrowUp") {
    event.preventDefault();
    focusOption(group, index + (event.key === "ArrowDown" ? 1 : -1));
  } else if (event.key === "Home" || event.key === "End") {
    event.preventDefault();
    const values = group === "hour" ? hours : minutes;
    focusOption(group, event.key === "Home" ? 0 : values.length - 1);
  } else if (event.key === "Escape") {
    event.preventDefault();
    closePicker();
    trigger.value?.focus();
  }
}

function onTriggerKeydown(event: KeyboardEvent) {
  if (event.key === "ArrowDown" && !open.value) {
    event.preventDefault();
    openPicker();
  } else if (event.key === "Escape" && open.value) {
    event.preventDefault();
    closePicker();
  }
}

function onDocumentPointerDown(event: PointerEvent) {
  if (!root.value?.contains(event.target as Node)) closePicker();
}

onMounted(() => document.addEventListener("pointerdown", onDocumentPointerDown));
onBeforeUnmount(() =>
  document.removeEventListener("pointerdown", onDocumentPointerDown),
);
</script>

<template>
  <div
    ref="root"
    class="p3-time-picker"
    :class="{ open, 'placement-bottom': placement === 'bottom' }"
  >
    <button
      ref="trigger"
      type="button"
      class="p3-picker-trigger"
      :data-testid="testId"
      :aria-label="label"
      :aria-expanded="open"
      :aria-controls="dialogId"
      aria-haspopup="dialog"
      @click="togglePicker"
      @keydown="onTriggerKeydown"
    >
      <span :class="{ placeholder: !displayValue }">
        {{ displayValue || placeholder }}
      </span>
      <i class="ph-thin ph-clock" aria-hidden="true"></i>
    </button>

    <button
      v-if="modelValue"
      type="button"
      class="p3-time-clear"
      aria-label="清除出生时间"
      data-testid="time-clear-inline"
      @click.stop="clearTime"
    >
      <i class="ph-thin ph-x" aria-hidden="true"></i>
    </button>

    <Transition name="p3-picker-panel">
      <section
        v-if="open"
        :id="dialogId"
        class="p3-picker-panel p3-time-panel"
        role="dialog"
        :aria-label="`${label}选择器`"
        @keydown.esc.stop.prevent="closePicker"
      >
        <header class="p3-time-heading">
          <div>
            <small>出生时刻</small>
            <strong>{{ draftHour }}:{{ draftMinute }}</strong>
          </div>
          <i class="ph-thin ph-moon-stars" aria-hidden="true"></i>
        </header>

        <div class="p3-time-columns">
          <section>
            <span>时</span>
            <div role="listbox" aria-label="小时">
              <button
                v-for="(hour, index) in hours"
                :key="hour"
                type="button"
                role="option"
                data-time-group="hour"
                :data-value="hour"
                :data-testid="`time-hour-${hour}`"
                :aria-selected="hour === draftHour"
                :class="{ selected: hour === draftHour }"
                @keydown="onOptionKeydown($event, 'hour', index)"
                @click="selectValue('hour', hour)"
              >
                {{ hour }}
              </button>
            </div>
          </section>
          <section>
            <span>分</span>
            <div role="listbox" aria-label="分钟">
              <button
                v-for="(minute, index) in minutes"
                :key="minute"
                type="button"
                role="option"
                data-time-group="minute"
                :data-value="minute"
                :data-testid="`time-minute-${minute}`"
                :aria-selected="minute === draftMinute"
                :class="{ selected: minute === draftMinute }"
                @keydown="onOptionKeydown($event, 'minute', index)"
                @click="selectValue('minute', minute)"
              >
                {{ minute }}
              </button>
            </div>
          </section>
        </div>

        <footer class="p3-picker-footer">
          <button type="button" data-testid="time-clear" @click="clearTime">
            清除
          </button>
          <span aria-hidden="true"></span>
          <button type="button" data-testid="time-confirm" @click="confirmTime">
            完成
          </button>
        </footer>
      </section>
    </Transition>
  </div>
</template>
