<script setup lang="ts">
import { computed, onMounted } from "vue";
import PageHeader from "@/components/common/PageHeader.vue";
import StatePanel from "@/components/common/StatePanel.vue";
import TrailChart from "@/components/TrailChart.vue";
import { formatChineseDate } from "@/services/zodiac";
import { useTrailStore } from "@/stores/trail";

const store = useTrailStore();
const entry = computed(() => store.selectedEntry);
const dimensionLabels = {
  love: "情感",
  work: "工作",
  wealth: "财运",
  social: "社交",
  inner: "内心",
} as const;

onMounted(() => store.load());
</script>

<template>
  <section class="secondary-page trail-page">
    <img class="star-background" src="/assets/star-field-background.png" alt="" />
    <PageHeader
      kicker="过去七天 · 星光回望"
      icon="ph-chart-line-up"
      title="时间经过，也留下了属于你的星光"
      subtitle="回看不是为了评判，只是重新听见自己走过的节奏。"
    />

    <StatePanel
      v-if="store.loading && !store.trail"
      icon="ph-chart-line-up"
      title="正在收拢星光"
      message="请稍候，过去的节奏正在排列。"
      loading
    />
    <StatePanel
      v-else-if="store.error && !store.trail"
      icon="ph-cloud-slash"
      title="暂时没有找到星迹"
      :message="store.error.message"
      action-label="重新读取"
      @action="store.load"
    />
    <StatePanel
      v-else-if="!store.trail?.entries.length"
      icon="ph-moon-stars"
      title="星迹还在等待第一束光"
      message="保存今天的心情后，这里会慢慢出现你的节奏。"
    />

    <template v-else-if="store.trail && entry">
      <section class="trail-summary" aria-label="七日星迹摘要">
        <article>
          <span>七日平均</span>
          <strong data-testid="trail-average">{{ store.trail.summary.average }}</strong>
          <small>比昨天多听见自己一点</small>
        </article>
        <article>
          <span>主要情绪</span>
          <strong>{{ store.trail.summary.primaryMood ?? "静" }}</strong>
          <small>出现了 {{ store.trail.summary.primaryMoodCount }} 次</small>
        </article>
        <article>
          <span>上升维度</span>
          <strong>+{{ store.trail.summary.risingDelta }}</strong>
          <small>{{ dimensionLabels[store.trail.summary.risingDimension] }}正在变亮</small>
        </article>
      </section>

      <div class="trail-layout">
        <section class="trend-card">
          <header>
            <div>
              <small>能量轨迹</small>
              <h2>{{ formatChineseDate(entry.date) }} · {{ entry.theme }}</h2>
            </div>
            <div class="trend-score">
              <strong>{{ entry.scores.overall }}</strong>
              <span>综合能量</span>
            </div>
          </header>
          <TrailChart
            :entries="store.trail.entries"
            :selected-date="store.selectedDate"
          />
          <div class="trail-days" role="tablist" aria-label="选择回望日期">
            <button
              v-for="item in store.trail.entries"
              :key="item.date"
              type="button"
              role="tab"
              :aria-selected="item.date === store.selectedDate"
              :class="{ selected: item.date === store.selectedDate }"
              :data-testid="`trail-day-${item.date}`"
              @click="store.select(item.date)"
            >
              <span>{{ item.weekday.replace("星期", "周") }}</span>
              <strong>{{ Number(item.date.slice(-2)) }}</strong>
            </button>
          </div>
        </section>

        <aside class="memory-card">
          <small>当日回望</small>
          <h2>留给此刻的你</h2>
          <div class="memory-mood">
            <span><i class="ph-duotone ph-diamond" aria-hidden="true"></i></span>
            <div>
              <small>那天的心情</small>
              <strong>{{ entry.mood ?? "未记录" }}</strong>
            </div>
          </div>
          <blockquote>{{ entry.echo }}</blockquote>
          <div class="dimension-grid">
            <div v-for="(label, key) in dimensionLabels" :key="key">
              <span>{{ label }}</span>
              <strong>{{ entry.scores.dimensions[key] }}</strong>
            </div>
          </div>
        </aside>
      </div>
    </template>
  </section>
</template>
