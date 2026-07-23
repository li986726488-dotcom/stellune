<script setup lang="ts">
import { computed, onMounted } from "vue";
import FortuneRadar from "@/components/today/FortuneRadar.vue";
import StatePanel from "@/components/common/StatePanel.vue";
import { formatChineseDate } from "@/services/zodiac";
import { useDailyReadingStore } from "@/stores/daily-reading";
import type { Mood } from "@/types";

const store = useDailyReadingStore();

const moods: Array<{ name: Mood; icon: string }> = [
  { name: "开心", icon: "ph-sun-horizon" },
  { name: "平静", icon: "ph-diamond" },
  { name: "迷茫", icon: "ph-spiral" },
  { name: "疲惫", icon: "ph-cloud-moon" },
  { name: "低落", icon: "ph-moon-stars" },
];

const reading = computed(() => store.reading);

onMounted(() => store.load());
</script>

<template>
  <section class="today-page">
    <img
      class="star-background"
      src="/assets/star-field-background.png"
      alt=""
      aria-hidden="true"
    />

    <StatePanel
      v-if="store.loading && !reading"
      icon="ph-stars"
      title="星图正在展开"
      message="正在读取今日的星相与节奏。"
      loading
    />
    <StatePanel
      v-else-if="store.error && !reading"
      icon="ph-cloud-slash"
      title="今天的星光暂时迟到"
      :message="store.error.message"
      action-label="重新读取"
      @action="store.load(true)"
    />

    <template v-else-if="reading">
      <div
        v-if="store.loading || store.requiresRecalibration || store.error"
        class="today-calibration-status"
        :class="{ error: store.error }"
        role="status"
      >
        <i
          class="ph-thin"
          :class="store.error ? 'ph-warning-circle' : 'ph-sparkle'"
          aria-hidden="true"
        ></i>
        <span v-if="store.loading">正在按新资料校准今日简报…</span>
        <span v-else-if="store.error">资料已更新，当前显示校准前版本。</span>
        <span v-else>当前简报基于更新前资料。</span>
        <button
          v-if="!store.loading"
          type="button"
          data-testid="retry-recalibration"
          @click="store.load(true)"
        >
          重新校准
        </button>
      </div>
      <div class="today-main">
        <header class="today-header">
          <p class="date-line">
            {{ formatChineseDate(reading.date) }}
            <span aria-hidden="true">·</span>
            {{ reading.weekday }}
          </p>
          <p class="today-zodiac">
            {{ reading.zodiac.name }}
            <i
              class="mdi"
              :class="`mdi-zodiac-${reading.zodiac.slug}`"
              aria-hidden="true"
            ></i>
          </p>
          <h1 data-testid="today-title">{{ reading.hero.title }}</h1>
          <p class="today-subtitle">{{ reading.hero.subtitle }}</p>
          <p class="today-theme">
            <i class="ph-thin ph-sparkle" aria-hidden="true"></i>
            今日主题 · {{ reading.hero.theme }}
          </p>
        </header>

        <FortuneRadar
          :scores="reading.scores.dimensions"
          :overall="reading.scores.overall"
        />
      </div>

      <aside class="reading-panel" aria-label="今日宇宙简报">
        <section>
          <h2>
            <i class="ph-thin ph-sparkle" aria-hidden="true"></i>
            今日宇宙简报
          </h2>
          <dl class="brief-list">
            <div v-for="brief in reading.briefs" :key="brief.key">
              <dt>{{ brief.label }}</dt>
              <dd>{{ brief.text }}</dd>
            </div>
          </dl>
        </section>

        <div class="panel-divider" aria-hidden="true"></div>

        <section>
          <h2>
            <i class="ph-thin ph-star-four" aria-hidden="true"></i>
            宇宙回声
          </h2>
          <blockquote>{{ reading.echo }}</blockquote>
        </section>

        <section class="mood-section">
          <p>记录此刻的心情</p>
          <div class="mood-list" role="group" aria-label="选择心情">
            <button
              v-for="mood in moods"
              :key="mood.name"
              class="mood-button"
              :class="{ selected: store.selectedMood === mood.name }"
              type="button"
              :aria-pressed="store.selectedMood === mood.name"
              :data-testid="`mood-${mood.name}`"
              @click="store.chooseMood(mood.name)"
            >
              <span><i class="ph-duotone" :class="mood.icon" aria-hidden="true"></i></span>
              <small>{{ mood.name }}</small>
            </button>
          </div>
          <button
            class="primary-button mood-save"
            type="button"
            :disabled="!store.selectedMood || store.savingMood"
            data-testid="save-mood"
            @click="store.saveMood"
          >
            <i class="ph-thin ph-star-four" aria-hidden="true"></i>
            {{ store.moodEntry ? "今日星迹已点亮" : "点亮今日星迹" }}
          </button>
        </section>
      </aside>

      <footer class="lucky-strip" aria-label="今日幸运提示">
        <div>
          <i class="ph-thin ph-drop" aria-hidden="true"></i>
          <span>幸运色 · {{ reading.lucky.color.value }}</span>
        </div>
        <div>
          <i class="ph-thin ph-number-circle-nine" aria-hidden="true"></i>
          <span>幸运数字 · {{ reading.lucky.number.value }}</span>
        </div>
        <div>
          <i class="ph-thin ph-clock" aria-hidden="true"></i>
          <span>幸运时段 · {{ reading.lucky.time.value }}</span>
        </div>
      </footer>
    </template>
  </section>
</template>
