import { createRouter, createWebHashHistory } from "vue-router";
import ExplorePage from "@/pages/ExplorePage.vue";
import OnboardingPage from "@/pages/OnboardingPage.vue";
import ProfilePage from "@/pages/ProfilePage.vue";
import TodayPage from "@/pages/TodayPage.vue";
import TrailPage from "@/pages/TrailPage.vue";

export const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    { path: "/", name: "today", component: TodayPage },
    { path: "/trail", name: "trail", component: TrailPage },
    { path: "/explore", name: "explore", component: ExplorePage },
    { path: "/profile", name: "profile", component: ProfilePage },
    {
      path: "/onboarding",
      name: "onboarding",
      component: OnboardingPage,
      meta: { standalone: true },
    },
    { path: "/:pathMatch(.*)*", redirect: "/" },
  ],
});
