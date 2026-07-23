<script setup lang="ts">
import { computed, onMounted, reactive, ref } from "vue";
import PageHeader from "@/components/common/PageHeader.vue";
import P3DatePicker from "@/components/common/P3DatePicker.vue";
import P3TimePicker from "@/components/common/P3TimePicker.vue";
import StatePanel from "@/components/common/StatePanel.vue";
import { formatChineseDate } from "@/services/zodiac";
import { useDailyReadingStore } from "@/stores/daily-reading";
import { useExploreStore } from "@/stores/explore";
import { useProfileStore } from "@/stores/profile";
import { useTrailStore } from "@/stores/trail";

const profileStore = useProfileStore();
const trailStore = useTrailStore();
const exploreStore = useExploreStore();
const dailyStore = useDailyReadingStore();
const editing = ref(false);
const confirmingChange = ref(false);
const editError = ref("");
const form = reactive({
  nickname: "",
  birthday: "",
  birthTime: "",
  birthCity: "",
});

const profile = computed(() => profileStore.profile);
const moodCount = computed(
  () => trailStore.trail?.entries.filter((entry) => entry.mood).length ?? 0,
);
const calculationChanged = computed(() => {
  if (!profile.value) return false;
  return (
    form.birthday !== profile.value.birthday ||
    (form.birthTime || null) !== profile.value.birthTime ||
    (form.birthCity.trim() || null) !== profile.value.birthCity
  );
});

function beginEdit() {
  if (!profile.value) return;
  form.nickname = profile.value.nickname;
  form.birthday = profile.value.birthday;
  form.birthTime = profile.value.birthTime ?? "";
  form.birthCity = profile.value.birthCity ?? "";
  confirmingChange.value = false;
  editing.value = true;
}

async function persistProfile() {
  editError.value = "";
  const shouldRecalibrate = calculationChanged.value;
  try {
    await profileStore.save({
      nickname: form.nickname,
      birthday: form.birthday,
      birthTime: form.birthTime || null,
      birthCity: form.birthCity || null,
    });
    editing.value = false;
    confirmingChange.value = false;
    if (shouldRecalibrate) {
      dailyStore.markNeedsRecalibration();
      void dailyStore.load(true).then((updatedReading) => {
        if (updatedReading && !updatedReading.source.stale) {
          void trailStore.load();
        }
      });
    }
  } catch {
    editError.value = profileStore.error?.message ?? "资料暂时没有保存成功。";
  }
}

async function save() {
  if (calculationChanged.value && !confirmingChange.value) {
    confirmingChange.value = true;
    return;
  }
  await persistProfile();
}

onMounted(async () => {
  if (!profileStore.profile) await profileStore.load();
  await Promise.all([trailStore.load(), exploreStore.load()]);
});
</script>

<template>
  <section class="secondary-page profile-page">
    <img class="star-background" src="/assets/star-field-background.png" alt="" />
    <PageHeader
      kicker="只属于你 · 也只留在此设备"
      icon="ph-user-circle"
      title="我的星迹档案"
      subtitle="在这里看见自己留下的光，也可以随时重新校准。"
    />

    <StatePanel
      v-if="profileStore.loading"
      icon="ph-user-circle"
      title="正在打开你的星迹档案"
      message="请稍候。"
      loading
    />

    <template v-else-if="profile">
      <section class="profile-hero">
        <span class="profile-avatar">
          <i class="ph-thin ph-user-focus" aria-hidden="true"></i>
        </span>
        <div class="profile-identity">
          <small>星图主人</small>
          <h2>{{ profile.nickname }}</h2>
          <p>{{ profile.zodiac.name }} · {{ formatChineseDate(profile.birthday) }}</p>
        </div>
        <div class="completeness">
          <div>
            <span>资料完整度</span>
            <strong>{{ profile.completeness }}%</strong>
          </div>
          <progress :value="profile.completeness" max="100"></progress>
          <small>完善资料，让解读更贴近你</small>
        </div>
        <button
          class="secondary-button"
          type="button"
          data-testid="edit-profile"
          @click="beginEdit"
        >
          <i class="ph-thin ph-pencil-simple" aria-hidden="true"></i>
          修改资料
        </button>
      </section>

      <div class="profile-grid">
        <section class="profile-card records-card">
          <header class="profile-card-heading">
            <div>
              <small>我的记录</small>
              <h2>被你点亮的时刻</h2>
            </div>
            <i class="ph-thin ph-chart-line-up" aria-hidden="true"></i>
          </header>
          <div class="record-stats">
            <article>
              <span>心情记录</span>
              <strong>{{ moodCount }}</strong>
              <small>次</small>
            </article>
            <article>
              <span>连续陪伴</span>
              <strong>{{ trailStore.trail?.entries.length ?? 0 }}</strong>
              <small>天</small>
            </article>
            <article>
              <span>今日一签</span>
              <strong>{{ exploreStore.fortune ? "已抽" : "未抽" }}</strong>
            </article>
          </div>
          <p>每一次停下来看看自己，都是值得被保存的一束光。</p>
        </section>

        <section class="profile-card archive-card">
          <header class="profile-card-heading">
            <div>
              <small>星图档案</small>
              <h2>关于你的星图</h2>
            </div>
            <i class="ph-thin ph-identification-card" aria-hidden="true"></i>
          </header>
          <dl>
            <div>
              <dt><i class="ph-thin ph-calendar-blank" aria-hidden="true"></i>生日</dt>
              <dd>{{ formatChineseDate(profile.birthday) }}</dd>
            </div>
            <div>
              <dt><i class="ph-thin ph-clock" aria-hidden="true"></i>出生时间</dt>
              <dd>{{ profile.birthTime || "未填写" }}</dd>
            </div>
            <div>
              <dt><i class="ph-thin ph-map-pin" aria-hidden="true"></i>出生城市</dt>
              <dd>{{ profile.birthCity || "未填写" }}</dd>
            </div>
          </dl>
        </section>
      </div>

      <div v-if="editing" class="modal-backdrop" role="presentation">
        <form
          class="profile-modal"
          data-testid="profile-form"
          @input="confirmingChange = false"
          @submit.prevent="save"
        >
          <header>
            <div>
              <small>重新校准</small>
              <h2>修改个人资料</h2>
            </div>
            <button type="button" aria-label="关闭" @click="editing = false">
              <i class="ph-thin ph-x" aria-hidden="true"></i>
            </button>
          </header>
          <label class="form-field">
            <span>称呼</span>
            <input v-model="form.nickname" maxlength="20" />
          </label>
          <div class="form-field">
            <span>生日</span>
            <P3DatePicker
              v-model="form.birthday"
              label="生日"
              placeholder="请选择生日"
              required
              test-id="profile-birthday"
              @update:model-value="confirmingChange = false"
            />
          </div>
          <div class="form-row">
            <div class="form-field">
              <span>出生时间</span>
              <P3TimePicker
                v-model="form.birthTime"
                label="出生时间"
                placement="bottom"
                test-id="profile-birth-time"
                @update:model-value="confirmingChange = false"
              />
            </div>
            <label class="form-field">
              <span>出生城市</span>
              <input v-model="form.birthCity" />
            </label>
          </div>
          <p v-if="editError" class="form-error">{{ editError }}</p>
          <div
            v-if="confirmingChange"
            class="profile-change-confirmation"
            role="alert"
            data-testid="profile-change-confirmation"
          >
            <i class="ph-thin ph-arrows-clockwise" aria-hidden="true"></i>
            <div>
              <strong>用新资料校准今天？</strong>
              <p>今日简报会重新生成，过去的星迹、心情和已抽签运不会改变。</p>
            </div>
            <button type="button" @click="confirmingChange = false">返回修改</button>
            <button
              class="confirm"
              type="button"
              data-testid="confirm-profile-recalibration"
              :disabled="profileStore.saving"
              @click="persistProfile"
            >
              确认保存
            </button>
          </div>
          <button
            v-else
            class="primary-button"
            type="submit"
            :disabled="profileStore.saving"
            data-testid="save-profile"
          >
            {{ calculationChanged ? "保存并校准今日简报" : "保存资料" }}
          </button>
        </form>
      </div>
    </template>
  </section>
</template>
