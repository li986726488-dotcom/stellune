<script setup lang="ts">
import {
  Chart,
  Filler,
  LineElement,
  PointElement,
  RadarController,
  RadialLinearScale,
  Tooltip,
} from "chart.js";
import type { Plugin } from "chart.js";
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from "vue";
import type { ScoreDimensions } from "@/types";

Chart.register(
  RadarController,
  RadialLinearScale,
  PointElement,
  LineElement,
  Filler,
  Tooltip,
);

const props = defineProps<{
  scores: ScoreDimensions;
  overall: number;
}>();

const canvas = ref<HTMLCanvasElement | null>(null);
let chart: Chart<"radar"> | null = null;

const values = computed(() => [
  props.scores.love,
  props.scores.work,
  props.scores.wealth,
  props.scores.social,
  props.scores.inner,
]);

const dimensionNames = ["情感", "工作", "财运", "社交", "内心"];
const labels = computed(() =>
  dimensionNames.map((label, index) => `${label} ${values.value[index]}`),
);

const ariaLabel = computed(() => labels.value.join("，"));

const fortunePointGlow: Plugin<"radar"> = {
  id: "fortunePointGlow",
  afterDatasetsDraw(chartInstance) {
    const meta = chartInstance.getDatasetMeta(0);
    const { ctx } = chartInstance;

    meta.data.forEach((point) => {
      const { x, y } = point;
      ctx.save();
      const glow = ctx.createRadialGradient(x, y, 1, x, y, 24);
      glow.addColorStop(0, "rgba(255, 247, 225, 0.98)");
      glow.addColorStop(0.16, "rgba(216, 201, 255, 0.9)");
      glow.addColorStop(0.48, "rgba(184, 160, 236, 0.3)");
      glow.addColorStop(1, "rgba(184, 160, 236, 0)");
      ctx.fillStyle = glow;
      ctx.beginPath();
      ctx.arc(x, y, 24, 0, Math.PI * 2);
      ctx.fill();

      ctx.strokeStyle = "rgba(255, 244, 221, 0.76)";
      ctx.lineWidth = 0.8;
      ctx.beginPath();
      ctx.moveTo(x - 11, y);
      ctx.lineTo(x + 11, y);
      ctx.moveTo(x, y - 11);
      ctx.lineTo(x, y + 11);
      ctx.stroke();
      ctx.restore();
    });
  },
};

const scorePointLabels: Plugin<"radar"> = {
  id: "scorePointLabels",
  afterDatasetsDraw(chartInstance) {
    const meta = chartInstance.getDatasetMeta(0);
    const scale = chartInstance.scales.r;
    const { ctx } = chartInstance;
    const scores = chartInstance.data.datasets[0].data as number[];

    ctx.save();
    ctx.fillStyle = "rgba(244, 236, 220, 0.94)";
    ctx.font = '500 12px "Noto Serif SC", "Songti SC", serif';
    ctx.textBaseline = "middle";

    meta.data.forEach((point, index) => {
      const dx = point.x - scale.xCenter;
      const dy = point.y - scale.yCenter;
      const length = Math.hypot(dx, dy) || 1;
      const x = point.x + (dx / length) * 18;
      const y = point.y + (dy / length) * 18;

      ctx.textAlign = Math.abs(dx) < 5 ? "center" : dx > 0 ? "left" : "right";
      ctx.fillText(`${dimensionNames[index]} ${scores[index]}`, x, y);
    });
    ctx.restore();
  },
};

function renderChart() {
  if (!canvas.value) return;
  chart?.destroy();
  chart = new Chart(canvas.value, {
    type: "radar",
    data: {
      labels: dimensionNames,
      datasets: [
        {
          data: values.value,
          borderColor: "rgba(226, 196, 143, 0.76)",
          backgroundColor: "rgba(184, 160, 236, 0.055)",
          borderWidth: 1.15,
          pointRadius: 4,
          pointHoverRadius: 5,
          pointBorderWidth: 1.4,
          pointBorderColor: "rgba(255, 244, 221, 0.92)",
          pointBackgroundColor: "#d3c6f4",
        },
      ],
    },
    options: {
      responsive: true,
      maintainAspectRatio: false,
      animation: false,
      layout: { padding: { top: 38, right: 72, bottom: 36, left: 72 } },
      scales: {
        r: {
          min: 0,
          max: 100,
          ticks: { display: false, stepSize: 20 },
          grid: { color: "rgba(130, 153, 193, 0.16)", circular: false },
          angleLines: { color: "rgba(130, 153, 193, 0.14)" },
          pointLabels: { display: false },
        },
      },
      plugins: {
        legend: { display: false },
        tooltip: { enabled: false },
      },
    },
    plugins: [fortunePointGlow, scorePointLabels],
  });
}

onMounted(renderChart);
watch(values, () => nextTick(renderChart));
onBeforeUnmount(() => chart?.destroy());
</script>

<template>
  <div class="fortune-radar" data-testid="fortune-radar">
    <canvas
      ref="canvas"
      role="img"
      :aria-label="ariaLabel"
    ></canvas>
    <div class="radar-score">
      <strong data-testid="overall-score">{{ overall }}</strong>
      <span>今日能量</span>
    </div>
  </div>
</template>
