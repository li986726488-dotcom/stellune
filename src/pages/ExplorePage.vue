<script setup lang="ts">
import { computed, nextTick, onMounted, ref } from "vue";
import PageHeader from "@/components/common/PageHeader.vue";
import P3Select, {
  type P3SelectOption,
} from "@/components/common/P3Select.vue";
import StatePanel from "@/components/common/StatePanel.vue";
import { useExploreStore } from "@/stores/explore";
import type { Mood, ZodiacSlug } from "@/types";

const store = useExploreStore();
const revealing = ref(false);
const cardRevealed = ref(false);
const cardSpinning = ref(false);
const cardRenderKey = ref(0);
const fortuneCard = ref<HTMLButtonElement | null>(null);
const resonanceSign = computed({
  get: () => store.partnerSign,
  set: (value: string) => {
    void store.loadCompatibility(value as ZodiacSlug);
  },
});
const resonanceOptions: P3SelectOption[] = [
  { value: "aries", label: "白羊座" },
  { value: "taurus", label: "金牛座" },
  { value: "gemini", label: "双子座" },
  { value: "cancer", label: "巨蟹座" },
  { value: "leo", label: "狮子座" },
  { value: "virgo", label: "处女座" },
  { value: "libra", label: "天秤座" },
  { value: "scorpio", label: "天蝎座" },
  { value: "sagittarius", label: "射手座" },
  { value: "capricorn", label: "摩羯座" },
  { value: "aquarius", label: "水瓶座" },
  { value: "pisces", label: "双鱼座" },
];
const visibleMoods: Mood[] = ["开心", "疲惫", "低落"];
const displayDate = computed(() => {
  const value = store.fortune?.date ? new Date(`${store.fortune.date}T12:00:00`) : new Date();
  return value.toLocaleDateString("zh-CN", { month: "long", day: "numeric" });
});
const fortuneNumberLabel = computed(() =>
  store.fortune
    ? `第 ${store.fortune.number} 签 · ${store.fortune.grade}`
    : "今日签运",
);

function waitForRevealAnimation(element: HTMLButtonElement) {
  return new Promise<void>((resolve) => {
    const timeoutId = window.setTimeout(finish, 2000);

    function finish() {
      window.clearTimeout(timeoutId);
      element.removeEventListener("animationend", handleAnimationEnd);
      resolve();
    }

    function handleAnimationEnd(event: AnimationEvent) {
      if (
        event.target === element &&
        ["fortune-reveal", "fortune-reveal-reduced"].includes(
          event.animationName,
        )
      ) {
        finish();
      }
    }

    element.addEventListener("animationend", handleAnimationEnd);
  });
}

async function showCardFront() {
  cardSpinning.value = false;
  cardRevealed.value = false;
  cardRenderKey.value += 1;
  await nextTick();
}

async function revealCard() {
  cardRevealed.value = true;
  cardRenderKey.value += 1;
  cardSpinning.value = true;
  await nextTick();
  const element = fortuneCard.value;
  if (element) await waitForRevealAnimation(element);
  cardSpinning.value = false;
}

async function drawFortune() {
  if (revealing.value) return;
  const hadFortune = Boolean(store.fortune);
  revealing.value = true;

  if (hadFortune) await showCardFront();

  const fortune = await store.draw();
  if (!fortune) {
    cardRevealed.value = hadFortune;
    cardRenderKey.value += 1;
    await nextTick();
    revealing.value = false;
    return;
  }

  await revealCard();
  revealing.value = false;
}

onMounted(async () => {
  await store.load();
  cardRevealed.value = Boolean(store.fortune);
});
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
            <small>随时可重抽</small>
            <h2>今日一签</h2>
          </div>
          <span>{{ displayDate }}</span>
        </header>

        <div class="fortune-card-scene">
          <button
            :key="cardRenderKey"
            ref="fortuneCard"
            class="fortune-card"
            :class="{
              revealed: cardRevealed,
              spinning: cardSpinning,
              drawing: store.drawing,
              revealing,
            }"
            type="button"
            data-testid="daily-fortune-card"
            :data-revealed="cardRevealed"
            :aria-label="store.fortune ? `${fortuneNumberLabel}：${store.fortune.title}` : '抽取今日一签'"
            :disabled="store.drawing || revealing || Boolean(store.fortune)"
            @click="drawFortune"
          >
            <span class="fortune-card-face fortune-card-front">
              <span class="fortune-card-emblem">
                <i class="ph-thin ph-scroll" aria-hidden="true"></i>
              </span>
              <strong>星轨百签</strong>
              <small>{{ revealing ? "星轨正在旋转…" : "轻触抽取今日签运" }}</small>
            </span>
            <span class="fortune-card-face fortune-card-back">
              <small data-testid="fortune-number">{{ fortuneNumberLabel }}</small>
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
          data-testid="draw-fortune"
          :disabled="store.drawing || revealing"
          @click="drawFortune"
        >
          <i
            class="ph-thin"
            :class="store.fortune ? 'ph-arrows-clockwise' : 'ph-sparkle'"
            aria-hidden="true"
          ></i>
          {{ revealing ? "星轨正在旋转…" : store.fortune ? "重新抽签" : "抽取今日一签" }}
        </button>
        <p class="fortune-lock">
          {{ revealing ? "签意正在靠近，请稍候" : store.fortune ? "每次重抽都会更新今日保存的签" : "轻触卡片或按钮，接住此刻的宇宙回音" }}
        </p>
      </section>

      <div class="explore-side">
        <article class="insight-card resonance-card">
          <header>
            <div>
              <small>太阳星座娱乐共鸣 · 非完整合盘</small>
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
          <div
            class="resonance-result"
            :class="{ updating: store.compatibilityLoading }"
            :aria-busy="store.compatibilityLoading"
          >
            <strong data-testid="compatibility-score">
              {{ store.compatibility?.score ?? "—" }}
            </strong>
            <div>
              <b data-testid="compatibility-title">
                {{ store.compatibility?.title ?? "正在辨认两颗星的节奏" }}
              </b>
              <span class="resonance-level">
                {{ store.compatibility?.level ?? "轻量共鸣" }}
              </span>
              <p>{{ store.compatibility?.summary ?? "选择一个星座，听听关系里的另一种可能。" }}</p>
            </div>
          </div>
          <p
            v-if="store.compatibilityError"
            class="insight-inline-error"
            role="alert"
            data-testid="compatibility-error"
          >
            {{ store.compatibilityError.message }} 已恢复上一次结果。
          </p>
        </article>

        <article class="insight-card emotion-card">
          <header>
            <div>
              <small>
                情绪说明书
                <template v-if="store.selectedMood"> · 当前：{{ store.selectedMood }}</template>
              </small>
              <h2>{{ store.compatibility?.primarySign.name ?? "你" }}此刻需要什么</h2>
            </div>
            <i class="ph-thin ph-book-open-text" aria-hidden="true"></i>
          </header>
          <div class="emotion-tabs">
            <button
              v-for="mood in visibleMoods"
              :key="mood"
              type="button"
              :class="{ selected: store.selectedMood === mood }"
              :aria-pressed="store.selectedMood === mood"
              :data-testid="`emotion-${mood}`"
              @click="store.selectMood(mood)"
            >
              {{ mood }}
            </button>
          </div>
          <div
            class="emotion-copy"
            :class="{ updating: store.emotionLoading }"
            :aria-busy="store.emotionLoading"
          >
            <h3 data-testid="emotion-guide-title">
              {{ store.emotionGuide?.title ?? "先听见此刻的自己" }}
            </h3>
            <p>{{ store.emotionGuide?.summary ?? "选择一种心情，宇宙简报会为此刻整理一份陪伴。" }}</p>
            <div v-if="store.emotionGuide" class="emotion-action">
              <i class="ph-thin ph-sparkle" aria-hidden="true"></i>
              <span>
                <small>此刻可以试试</small>
                <b data-testid="emotion-guide-action">{{ store.emotionGuide.action }}</b>
              </span>
            </div>
            <p
              v-if="store.emotionError"
              class="insight-inline-error"
              role="alert"
              data-testid="emotion-error"
            >
              {{ store.emotionError.message }} 已恢复上一次心情。
            </p>
          </div>
        </article>
      </div>
    </div>
  </section>
</template>
