import React from "react";
import ReactDOM from "react-dom/client";
import { PopupWindow } from "./windows/popup/PopupWindow";
import { SettingsWindow } from "./windows/settings/SettingsWindow";
import { UpdateWindow } from "./windows/update/UpdateWindow";
import "./styles/globals.css";

const AppRouter: React.FC = () => {
  const params = new URLSearchParams(window.location.search);
  const windowType = params.get("window") ?? "popup";

  if (windowType === "settings") {
    return <SettingsWindow />;
  }
  if (windowType === "update") {
    return <UpdateWindow />;
  }
  return <PopupWindow />;
};

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <AppRouter />
  </React.StrictMode>
);
