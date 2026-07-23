<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import PageHeader from "@/components/common/PageHeader.vue";
import P3Select, {
  type P3SelectOption,
} from "@/components/common/P3Select.vue";
import StatePanel from "@/components/common/StatePanel.vue";
import { useExploreStore } from "@/stores/explore";

const store = useExploreStore();
const revealing = ref(false);
const resonanceSign = ref("gemini");
const resonanceOptions: P3SelectOption[] = [
  { value: "gemini", label: "双子座" },
  { value: "aquarius", label: "水瓶座" },
  { value: "leo", label: "狮子座" },
];
const displayDate = computed(() => {
  const value = store.fortune?.date ? new Date(`${store.fortune.date}T12:00:00`) : new Date();
  return value.toLocaleDateString("zh-CN", { month: "long", day: "numeric" });
});

async function drawFortune() {
  if (store.fortune || revealing.value) return;
  revealing.value = true;
  const fortune = await store.draw();
  if (fortune && !window.matchMedia("(prefers-reduced-motion: reduce)").matches) {
    await new Promise((resolve) => window.setTimeout(resolve, 1600));
  }
  revealing.value = false;
}

onMounted(() => store.load());
</script>

<template>
  <section class="secondary-page explore-page">
    <img class="star-background" src="/assets/star-field-background.png" alt="" />
    <PageHeader
      kicker="轻轻探索 · 不急着得到标准答案"
      icon="ph-compass"
      title="沿着好奇，听见另一种答案"
      subtitle="给关系一点理解，也为今天抽取一支属于你的签。"
    />

    <StatePanel
      v-if="store.loading && !store.initialized"
      icon="ph-compass"
      title="答案正在靠近"
      message="请稍候，今天的签正在安静落下。"
      loading
    />

    <div v-else class="explore-grid">
      <section class="fortune-section">
        <header>
          <div>
            <small>一天一次</small>
            <h2>今日一签</h2>
          </div>
          <span>{{ displayDate }}</span>
        </header>

        <div class="fortune-card-scene">
          <button
            class="fortune-card"
            :class="{
              revealed: Boolean(store.fortune),
              drawing: store.drawing,
              revealing,
            }"
            type="button"
            data-testid="daily-fortune-card"
            :data-revealed="Boolean(store.fortune)"
            :aria-label="store.fortune ? `今日${store.fortune.grade}：${store.fortune.title}` : '抽取今日一签'"
            :disabled="store.drawing || revealing"
            @click="drawFortune"
          >
            <span class="fortune-card-face fortune-card-front">
              <span class="fortune-card-emblem">
                <i class="ph-thin ph-scroll" aria-hidden="true"></i>
              </span>
              <strong>宇宙灵签</strong>
              <small>{{ revealing ? "星轨正在旋转…" : "轻触抽取今日签运" }}</small>
            </span>
            <span class="fortune-card-face fortune-card-back">
              <small>{{ store.fortune?.grade ?? "今日签运" }}</small>
              <strong data-testid="fortune-title">{{ store.fortune?.title ?? "静候星光" }}</strong>
              <em>{{ store.fortune?.verse ?? "签语正在靠近" }}</em>
              <span>{{ store.fortune?.interpretation ?? "轻触卡片，接住今天的宇宙回音。" }}</span>
              <b>今日建议 · {{ store.fortune?.advice ?? "先听一听内心的声音" }}</b>
            </span>
          </button>
        </div>

        <button
          class="draw-action"
          type="button"
          :disabled="Boolean(store.fortune) || store.drawing || revealing"
          @click="drawFortune"
        >
          <i class="ph-thin ph-sparkle" aria-hidden="true"></i>
          {{ revealing ? "星轨正在旋转…" : store.fortune ? "今日签运已揭晓" : "抽取今日一签" }}
        </button>
        <p class="fortune-lock">
          {{ revealing ? "签意正在靠近，请稍候" : store.fortune ? "今日签运已揭晓 · 明日再来" : "每个自然日只能抽取一次" }}
        </p>
      </section>

      <div class="explore-side">
        <article class="insight-card resonance-card">
          <header>
            <div>
              <small>关系共鸣</small>
              <h2>当两颗星靠近</h2>
            </div>
            <i class="ph-thin ph-users-three" aria-hidden="true"></i>
          </header>
          <P3Select
            v-model="resonanceSign"
            class="resonance-select"
            label="选择对方星座"
            :options="resonanceOptions"
            data-testid="resonance-sign"
          />
          <div class="resonance-result">
            <strong>88</strong>
            <div>
              <b>风与风的默契</b>
              <p>你们都懂得给彼此空间，真诚表达会让关系更轻盈。</p>
            </div>
          </div>
        </article>

        <article class="insight-card emotion-card">
          <header>
            <div>
              <small>情绪说明书</small>
              <h2>天秤座此刻需要什么</h2>
            </div>
            <i class="ph-thin ph-book-open-text" aria-hidden="true"></i>
          </header>
          <div class="emotion-tabs">
            <button type="button" class="selected">开心</button>
            <button type="button">疲惫</button>
            <button type="button">低落</button>
          </div>
          <h3>把快乐分享给喜欢的人</h3>
          <p>你的好心情会在交流里变得更明亮，但也别忘了留一点只属于自己的满足。</p>
        </article>
      </div>
    </div>
  </section>
</template>
