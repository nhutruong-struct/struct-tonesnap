import React from "react";
import ReactDOM from "react-dom/client";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import Popup from "./popup/Popup";
import SettingsPage from "./settings/SettingsPage";
import "./styles/index.css";

// One bundle serves both windows; the window label decides what to render.
const label = getCurrentWebviewWindow().label;
document.documentElement.dataset.window = label;

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>{label === "popup" ? <Popup /> : <SettingsPage />}</React.StrictMode>,
);
