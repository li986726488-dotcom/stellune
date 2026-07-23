<script setup lang="ts">
import { computed, ref } from "vue";
import { useRouter } from "vue-router";
import { zodiacFromBirthday } from "@/services/zodiac";
import { useAppStore } from "@/stores/app";
import { useDailyReadingStore } from "@/stores/daily-reading";
import { useProfileStore } from "@/stores/profile";

const router = useRouter();
const appStore = useAppStore();
const profileStore = useProfileStore();
const dailyStore = useDailyReadingStore();

const birthday = ref("");
const nickname = ref("");
const birthTime = ref("");
const birthCity = ref("");
const showMore = ref(false);
const formError = ref("");

const zodiac = computed(() =>
  birthday.value ? zodiacFromBirthday(birthday.value) : null,
);

async function submit() {
  formError.value = "";
  if (!birthday.value) {
    formError.value = "请先告诉我们你的生日。";
    return;
  }

  try {
    const profile = await profileStore.save({
      birthday: birthday.value,
      nickname: nickname.value.trim() || undefined,
      birthTime: birthTime.value || null,
      birthCity: birthCity.value.trim() || null,
    });
    appStore.setProfileReady(profile);
    dailyStore.invalidate();
    await router.replace({ name: "today" });
  } catch {
    formError.value = profileStore.error?.message ?? "资料暂时没有保存成功。";
  }
}
</script>

<template>
  <section class="onboarding-page">
    <img
      class="star-background"
      src="/assets/star-field-background.png"
      alt=""
      aria-hidden="true"
    />

    <header class="onboarding-brand">
      <strong>星迹</strong>
      <span>你的第一束星光</span>
    </header>

    <div class="onboarding-copy">
      <p class="onboarding-step">FIRST LIGHT · 01</p>
      <h1>从你的生日开始，<br />认识今天的自己</h1>
      <p>
        生日会帮助我们找到你的太阳星座，并生成只属于你的每日宇宙简报。
      </p>
      <ul>
        <li><i class="ph-thin ph-device-mobile" aria-hidden="true"></i>资料只保存在此设备</li>
        <li><i class="ph-thin ph-sparkle" aria-hidden="true"></i>不需要注册或登录</li>
      </ul>
    </div>

    <form class="onboarding-card" data-testid="onboarding-form" @submit.prevent="submit">
      <div class="card-heading">
        <span class="card-icon"><i class="ph-thin ph-calendar-star" aria-hidden="true"></i></span>
        <div>
          <small>建立你的星图</small>
          <h2>先告诉我你的生日</h2>
        </div>
      </div>

      <label class="form-field">
        <span>生日 <em>必须</em></span>
        <input
          v-model="birthday"
          type="date"
          required
          data-testid="birthday-input"
          aria-describedby="birthday-hint"
        />
      </label>

      <div class="zodiac-preview" id="birthday-hint">
        <span class="zodiac-preview-icon">
          <i
            v-if="zodiac"
            class="mdi"
            :class="`mdi-zodiac-${zodiac.slug}`"
            aria-hidden="true"
          ></i>
          <i v-else class="ph-thin ph-sparkle" aria-hidden="true"></i>
        </span>
        <div>
          <small>你的太阳星座</small>
          <strong>{{ zodiac?.name ?? "等待生日" }}</strong>
        </div>
      </div>

      <button
        class="more-toggle"
        type="button"
        :aria-expanded="showMore"
        data-testid="more-profile-toggle"
        @click="showMore = !showMore"
      >
        <span>完善更多资料</span>
        <small>可选</small>
        <i
          class="ph-thin ph-caret-down"
          :class="{ rotated: showMore }"
          aria-hidden="true"
        ></i>
      </button>

      <div v-if="showMore" class="optional-fields">
        <label class="form-field">
          <span>称呼</span>
          <input v-model="nickname" type="text" maxlength="20" placeholder="星际旅人" />
        </label>
        <div class="form-row">
          <label class="form-field">
            <span>出生时间</span>
            <input v-model="birthTime" type="time" />
          </label>
          <label class="form-field">
            <span>出生城市</span>
            <input v-model="birthCity" type="text" maxlength="30" placeholder="上海" />
          </label>
        </div>
      </div>

      <p v-if="formError" class="form-error" role="alert">{{ formError }}</p>

      <button
        class="primary-button"
        type="submit"
        :disabled="!birthday || profileStore.saving"
        data-testid="onboarding-submit"
      >
        <i class="ph-thin ph-star-four" aria-hidden="true"></i>
        {{ profileStore.saving ? "正在点亮…" : "开启我的星图" }}
      </button>
      <p class="privacy-note">
        <i class="ph-thin ph-lock-key" aria-hidden="true"></i>
        不上传云端，之后可以在「我的」中修改
      </p>
    </form>
  </section>
</template>
