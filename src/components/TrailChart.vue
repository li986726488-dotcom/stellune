<script setup lang="ts">
import {
  CategoryScale,
  Chart,
  Filler,
  LinearScale,
  LineController,
  LineElement,
  PointElement,
  Tooltip,
} from "chart.js";
import { nextTick, onBeforeUnmount, onMounted, ref, watch } from "vue";
import type { TrailEntry } from "@/types";

Chart.register(
  CategoryScale,
  LinearScale,
  LineController,
  LineElement,
  PointElement,
  Filler,
  Tooltip,
);

const props = defineProps<{
  entries: TrailEntry[];
  selectedDate: string | null;
}>();

const canvas = ref<HTMLCanvasElement | null>(null);
let chart: Chart<"line"> | null = null;

function renderChart() {
  if (!canvas.value) return;
  chart?.destroy();
  chart = new Chart(canvas.value, {
    type: "line",
    data: {
      labels: props.entries.map((entry) => entry.weekday),
      datasets: [
        {
          data: props.entries.map((entry) => entry.scores.overall),
          borderColor: "rgba(222, 198, 143, 0.86)",
          backgroundColor: "rgba(170, 150, 223, 0.08)",
          borderWidth: 1.6,
          pointRadius: props.entries.map((entry) =>
            entry.date === props.selectedDate ? 5 : 3,
          ),
          pointBorderColor: "rgba(255, 244, 221, 0.92)",
          pointBackgroundColor: "rgba(205, 189, 255, 0.95)",
          tension: 0.34,
          fill: true,
        },
      ],
    },
    options: {
      responsive: true,
      maintainAspectRatio: false,
      animation: false,
      scales: {
        x: {
          grid: { display: false },
          border: { display: false },
          ticks: {
            color: "rgba(164, 177, 202, 0.72)",
            font: { size: 11 },
          },
        },
        y: {
          min: 60,
          max: 90,
          ticks: { display: false },
          border: { display: false },
          grid: { color: "rgba(128, 151, 190, 0.12)" },
        },
      },
      plugins: {
        legend: { display: false },
        tooltip: { enabled: false },
      },
    },
  });
}

onMounted(renderChart);
watch(
  () => [props.entries, props.selectedDate],
  () => nextTick(renderChart),
  { deep: true },
);
onBeforeUnmount(() => chart?.destroy());
</script>

<template>
  <div class="trail-chart">
    <canvas
      ref="canvas"
      role="img"
      aria-label="最近七天综合能量趋势"
    ></canvas>
  </div>
</template>
