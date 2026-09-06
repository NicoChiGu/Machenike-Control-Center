import { createApp } from "vue";
import "./style.css";
import App from "./App.vue";
import OsdOverlay from "./components/OsdOverlay.vue";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";

let isOsd = false;
try {
  const currentWin = getCurrentWebviewWindow();
  isOsd = currentWin.label === "osd";
} catch {
  isOsd =
    window.location.hash === "#osd" ||
    window.location.search.includes("window=osd");
}

if (isOsd) {
  document.documentElement.classList.add("is-osd");
  document.body.classList.add("is-osd");
  createApp(OsdOverlay).mount("#app");
} else {
  createApp(App).mount("#app");
}
