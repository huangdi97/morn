import { useState } from "react";
import WelcomeNoKey from "./WelcomeNoKey";
import WelcomeReady from "./WelcomeReady";
import WelcomeError from "./WelcomeError";

interface WelcomeGuideProps {
  onDismiss: () => void;
  onSend?: (text: string) => void;
}

export default function WelcomeGuide({ onDismiss, onSend }: WelcomeGuideProps) {
  const [state] = useState<"no_key" | "ready" | "error">("no_key");

  switch (state) {
    case "ready":
      return <WelcomeReady onSend={onSend || (() => {})} />;
    case "error":
      return <WelcomeError onDismiss={onDismiss} />;
    case "no_key":
    default:
      return <WelcomeNoKey onDismiss={onDismiss} />;
  }
}