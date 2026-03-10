import { createApp } from "vue";
import { createPinia } from "pinia";
import router from "./router";
import App from "./App.vue";
import "./styles/main.css";
import { getRuntimeCapabilities } from "@/platform";

const app = createApp(App);

if (import.meta.env.DEV) {
  console.info("[runtime-capabilities]", getRuntimeCapabilities());
}

app.use(createPinia());
app.use(router);

app.mount("#app");