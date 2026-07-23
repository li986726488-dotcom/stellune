import { createApp } from "vue";
import { createPinia } from "pinia";
import App from "@/App.vue";
import { router } from "@/router";
import { initializeCommandClient } from "@/services/commands";
import "@/styles/phosphor-thin.css";
import "@/styles/phosphor-duotone.css";
import "@/styles/mdi-zodiac.css";
import "@/styles/tokens.css";
import "@/styles/global.css";

async function bootstrap() {
  await initializeCommandClient();
  createApp(App).use(createPinia()).use(router).mount("#app");
}

void bootstrap();
