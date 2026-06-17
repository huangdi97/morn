import { useEffect, useState } from "react";
import { getApiConfig } from "../Settings";
import WelcomeNoKey from "./WelcomeNoKey";
import WelcomeReady from "./WelcomeReady";
import WelcomeError from "./WelcomeError";

interface WelcomeGuideProps {
  onDismiss: () => void;
  onSend?: (text: string) => void;
}

export default function WelcomeGuide({ onDismiss, onSend }: WelcomeGuideProps) {
  const [state, setState] = useState<"no_key" | "ready" | "error">("no_key");

  useEffect(() => {
    try {
      const config = getApiConfig();
      setState(config.apiKey ? "ready" : "no_key");
    } catch {
      setState("error");
    }
  }, []);

  switch (state) {
    case "ready":
      return <WelcomeReady onSend={onSend || (() => {})} />;
    case "error":
      return <WelcomeError onDismiss={onDismiss} />;
    case "no_key":
    default:
      return <WelcomeNoKey onDismiss={onDismiss} onReady={() => setState("ready")} />;
  }
}