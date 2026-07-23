<script setup lang="ts">
import {
  computed,
  nextTick,
  onBeforeUnmount,
  onMounted,
  ref,
  useId,
  watch,
} from "vue";

type CalendarView = "days" | "years" | "months";

interface DateParts {
  year: number;
  month: number;
  day: number;
}

interface CalendarCell {
  key: string;
  day: number;
  inMonth: boolean;
  disabled: boolean;
  isToday: boolean;
  isSelected: boolean;
}

const props = withDefaults(
  defineProps<{
    modelValue: string;
    label: string;
    placeholder?: string;
    required?: boolean;
    min?: string;
    max?: string;
    testId?: string;
    describedBy?: string;
  }>(),
  {
    placeholder: "请选择日期",
    required: false,
    min: "",
    max: "",
    testId: undefined,
    describedBy: undefined,
  },
);

const emit = defineEmits<{
  "update:modelValue": [value: string];
}>();

const weekLabels = ["日", "一", "二", "三", "四", "五", "六"];
const monthLabels = [
  "一月",
  "二月",
  "三月",
  "四月",
  "五月",
  "六月",
  "七月",
  "八月",
  "九月",
  "十月",
  "十一月",
  "十二月",
];
const root = ref<HTMLElement | null>(null);
const trigger = ref<HTMLButtonElement | null>(null);
const open = ref(false);
const view = ref<CalendarView>("days");
const displayYear = ref(2000);
const displayMonth = ref(0);
const yearPageStart = ref(1992);
const activeDate = ref("");
const componentId = useId();
const dialogId = `${componentId}-dialog`;

function toKey(date: Date) {
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, "0");
  const day = String(date.getDate()).padStart(2, "0");
  return `${year}-${month}-${day}`;
}

function parseKey(value: string): DateParts | null {
  const match = /^(\d{4})-(\d{2})-(\d{2})$/.exec(value);
  if (!match) return null;
  const year = Number(match[1]);
  const month = Number(match[2]);
  const day = Number(match[3]);
  const date = new Date(year, month - 1, day);
  if (
    date.getFullYear() !== year ||
    date.getMonth() !== month - 1 ||
    date.getDate() !== day
  ) {
    return null;
  }
  return { year, month: month - 1, day };
}

function fromParts(year: number, month: number, day: number) {
  return toKey(new Date(year, month, day));
}

const today = new Date();
const todayKey = toKey(today);
const effectiveMax = computed(() => props.max || todayKey);
const selectedParts = computed(() => parseKey(props.modelValue));
const displayValue = computed(() =>
  selectedParts.value
    ? `${selectedParts.value.year}/${String(selectedParts.value.month + 1).padStart(2, "0")}/${String(selectedParts.value.day).padStart(2, "0")}`
    : "",
);
const headerLabel = computed(() => {
  if (view.value === "years") {
    return `${yearPageStart.value}—${yearPageStart.value + 11}`;
  }
  if (view.value === "months") return `${displayYear.value}年`;
  return `${displayYear.value}年${String(displayMonth.value + 1).padStart(2, "0")}月`;
});
const years = computed(() =>
  Array.from({ length: 12 }, (_, index) => yearPageStart.value + index),
);
const cells = computed<CalendarCell[]>(() => {
  const firstWeekday = new Date(
    displayYear.value,
    displayMonth.value,
    1,
  ).getDay();
  const start = new Date(
    displayYear.value,
    displayMonth.value,
    1 - firstWeekday,
  );
  return Array.from({ length: 42 }, (_, index) => {
    const date = new Date(
      start.getFullYear(),
      start.getMonth(),
      start.getDate() + index,
    );
    const key = toKey(date);
    return {
      key,
      day: date.getDate(),
      inMonth: date.getMonth() === displayMonth.value,
      disabled:
        (props.min.length > 0 && key < props.min) ||
        (effectiveMax.value.length > 0 && key > effectiveMax.value),
      isToday: key === todayKey,
      isSelected: key === props.modelValue,
    };
  });
});

function syncDisplay() {
  const parts = selectedParts.value ?? {
    year: today.getFullYear(),
    month: today.getMonth(),
    day: today.getDate(),
  };
  displayYear.value = parts.year;
  displayMonth.value = parts.month;
  yearPageStart.value = Math.floor(parts.year / 12) * 12;
  activeDate.value = props.modelValue || todayKey;
  view.value = "days";
}

function openPicker() {
  syncDisplay();
  open.value = true;
}

function closePicker() {
  open.value = false;
  view.value = "days";
}

function togglePicker() {
  if (open.value) closePicker();
  else openPicker();
}

function shiftMonth(offset: number) {
  const target = new Date(displayYear.value, displayMonth.value + offset, 1);
  displayYear.value = target.getFullYear();
  displayMonth.value = target.getMonth();
}

function navigatePrevious() {
  if (view.value === "days") shiftMonth(-1);
  else if (view.value === "months") displayYear.value -= 1;
  else yearPageStart.value -= 12;
}

function navigateNext() {
  if (view.value === "days") shiftMonth(1);
  else if (view.value === "months") displayYear.value += 1;
  else yearPageStart.value += 12;
}

function toggleHeaderView() {
  if (view.value === "days") {
    yearPageStart.value = Math.floor(displayYear.value / 12) * 12;
    view.value = "years";
  } else if (view.value === "months") {
    yearPageStart.value = Math.floor(displayYear.value / 12) * 12;
    view.value = "years";
  }
}

function selectYear(year: number) {
  displayYear.value = year;
  view.value = "months";
}

function selectMonth(month: number) {
  displayMonth.value = month;
  view.value = "days";
}

function selectDate(cell: CalendarCell) {
  if (cell.disabled) return;
  emit("update:modelValue", cell.key);
  closePicker();
  trigger.value?.focus();
}

function clearDate() {
  emit("update:modelValue", "");
  closePicker();
  trigger.value?.focus();
}

function selectToday() {
  const cell = cells.value.find((item) => item.key === todayKey);
  if (cell && !cell.disabled) {
    selectDate(cell);
    return;
  }
  if (
    (!props.min || todayKey >= props.min) &&
    (!effectiveMax.value || todayKey <= effectiveMax.value)
  ) {
    emit("update:modelValue", todayKey);
    closePicker();
    trigger.value?.focus();
  }
}

async function focusDate(key: string) {
  const parts = parseKey(key);
  if (!parts) return;
  if (
    (props.min && key < props.min) ||
    (effectiveMax.value && key > effectiveMax.value)
  ) {
    return;
  }
  displayYear.value = parts.year;
  displayMonth.value = parts.month;
  activeDate.value = key;
  await nextTick();
  root.value
    ?.querySelector<HTMLButtonElement>(`[data-date="${key}"]`)
    ?.focus();
}

function moveActive(days: number) {
  const parts =
    parseKey(activeDate.value) ??
    selectedParts.value ?? {
      year: today.getFullYear(),
      month: today.getMonth(),
      day: today.getDate(),
    };
  const target = new Date(parts.year, parts.month, parts.day + days);
  void focusDate(toKey(target));
}

function moveActiveMonth(offset: number) {
  const parts =
    parseKey(activeDate.value) ??
    selectedParts.value ?? {
      year: today.getFullYear(),
      month: today.getMonth(),
      day: today.getDate(),
    };
  const lastDay = new Date(parts.year, parts.month + offset + 1, 0).getDate();
  const target = new Date(
    parts.year,
    parts.month + offset,
    Math.min(parts.day, lastDay),
  );
  void focusDate(toKey(target));
}

function onDayKeydown(event: KeyboardEvent) {
  const offsets: Record<string, number> = {
    ArrowLeft: -1,
    ArrowRight: 1,
    ArrowUp: -7,
    ArrowDown: 7,
  };
  if (event.key in offsets) {
    event.preventDefault();
    moveActive(offsets[event.key]);
  } else if (event.key === "PageUp" || event.key === "PageDown") {
    event.preventDefault();
    moveActiveMonth(event.key === "PageUp" ? -1 : 1);
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

watch(
  () => props.modelValue,
  () => {
    if (!open.value) syncDisplay();
  },
);

onMounted(() => document.addEventListener("pointerdown", onDocumentPointerDown));
onBeforeUnmount(() =>
  document.removeEventListener("pointerdown", onDocumentPointerDown),
);
</script>

<template>
  <div ref="root" class="p3-date-picker" :class="{ open }">
    <button
      ref="trigger"
      type="button"
      class="p3-picker-trigger"
      :data-testid="testId"
      :aria-label="label"
      :aria-required="required"
      :aria-describedby="describedBy"
      :aria-expanded="open"
      :aria-controls="dialogId"
      aria-haspopup="dialog"
      @click="togglePicker"
      @keydown="onTriggerKeydown"
    >
      <span :class="{ placeholder: !displayValue }">
        {{ displayValue || placeholder }}
      </span>
      <i class="ph-thin ph-calendar-blank" aria-hidden="true"></i>
    </button>

    <Transition name="p3-picker-panel">
      <section
        v-if="open"
        :id="dialogId"
        class="p3-picker-panel p3-calendar-panel"
        role="dialog"
        :aria-label="`${label}选择器`"
        @keydown.esc.stop.prevent="closePicker"
      >
        <header class="p3-picker-header">
          <button
            type="button"
            aria-label="上一页"
            data-testid="calendar-previous"
            @click="navigatePrevious"
          >
            <i class="ph-thin ph-caret-left" aria-hidden="true"></i>
          </button>
          <button
            type="button"
            class="p3-picker-title"
            data-testid="calendar-title"
            :aria-label="view === 'years' ? headerLabel : `切换年份，当前${headerLabel}`"
            @click="toggleHeaderView"
          >
            {{ headerLabel }}
            <i
              v-if="view !== 'years'"
              class="ph-thin ph-caret-down"
              aria-hidden="true"
            ></i>
          </button>
          <button
            type="button"
            aria-label="下一页"
            data-testid="calendar-next"
            @click="navigateNext"
          >
            <i class="ph-thin ph-caret-right" aria-hidden="true"></i>
          </button>
        </header>

        <template v-if="view === 'days'">
          <div class="p3-calendar-weekdays" aria-hidden="true">
            <span v-for="weekday in weekLabels" :key="weekday">{{ weekday }}</span>
          </div>
          <div class="p3-calendar-days" role="grid" :aria-label="headerLabel">
            <button
              v-for="cell in cells"
              :key="cell.key"
              type="button"
              role="gridcell"
              :data-date="cell.key"
              :data-testid="`calendar-day-${cell.key}`"
              :aria-label="cell.key"
              :aria-selected="cell.isSelected"
              :disabled="cell.disabled"
              :tabindex="cell.key === activeDate ? 0 : -1"
              :class="{
                adjacent: !cell.inMonth,
                today: cell.isToday,
                selected: cell.isSelected,
              }"
              @focus="activeDate = cell.key"
              @keydown="onDayKeydown"
              @click="selectDate(cell)"
            >
              {{ cell.day }}
            </button>
          </div>
        </template>

        <div v-else-if="view === 'years'" class="p3-calendar-years">
          <button
            v-for="year in years"
            :key="year"
            type="button"
            :data-testid="`calendar-year-${year}`"
            :class="{ selected: year === displayYear }"
            @click="selectYear(year)"
          >
            {{ year }}
          </button>
        </div>

        <div v-else class="p3-calendar-months">
          <button
            v-for="(month, index) in monthLabels"
            :key="month"
            type="button"
            :data-testid="`calendar-month-${index + 1}`"
            :class="{ selected: index === displayMonth }"
            @click="selectMonth(index)"
          >
            {{ month }}
          </button>
        </div>

        <footer class="p3-picker-footer">
          <button type="button" data-testid="calendar-clear" @click="clearDate">
            清除
          </button>
          <span aria-hidden="true"></span>
          <button type="button" data-testid="calendar-today" @click="selectToday">
            今天
          </button>
        </footer>
      </section>
    </Transition>
  </div>
</template>
