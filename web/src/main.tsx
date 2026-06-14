import { registerPlugin } from "./slots/SlotRegistry";
import { useState } from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import WelcomeGuide from "./welcome/WelcomeGuide";
import "./App.css";

function Root() {
  const [welcomed, setWelcomed] = useState(() => localStorage.getItem("morn_welcomed") === "true");

  if (!welcomed) {
    return <WelcomeGuide onDismiss={() => setWelcomed(true)} />;
  }

  return <App />;
}

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <Root />
);

function DemoFooter() {
  return (
    <footer style={{ textAlign: "center", padding: 8, fontSize: 12, color: "#888" }}>
      ⚡ Slot Plugin — Morn Desktop
    </footer>
  );
}

registerPlugin({
  id: "demo-footer",
  position: "footer",
  label: "Demo Footer",
  component: DemoFooter,
  order: 0,
});