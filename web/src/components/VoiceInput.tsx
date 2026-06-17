import { useState, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";

interface VoiceInputProps {
  onTranscribed: (text: string) => void;
}

export default function VoiceInput({ onTranscribed }: VoiceInputProps) {
  const [recording, setRecording] = useState(false);
  const [processing, setProcessing] = useState(false);
  const mediaRecorder = useRef<MediaRecorder | null>(null);
  const chunks = useRef<Blob[]>([]);

  const startRecording = async () => {
    try {
      const stream = await navigator.mediaDevices.getUserMedia({ audio: true });
      const recorder = new MediaRecorder(stream, { mimeType: "audio/webm" });
      mediaRecorder.current = recorder;
      chunks.current = [];

      recorder.ondataavailable = (e) => {
        if (e.data.size > 0) chunks.current.push(e.data);
      };

      recorder.onstop = async () => {
        setProcessing(true);
        const blob = new Blob(chunks.current, { type: "audio/webm" });

        // Write file to temp and transcribe
        try {
          const text: string = await invoke("transcribe_audio", {
            path: URL.createObjectURL(blob),
          });
          onTranscribed(text);
        } catch (e) {
          console.error("Transcription failed:", e);
        }

        stream.getTracks().forEach((t) => t.stop());
        setProcessing(false);
      };

      recorder.start();
      setRecording(true);
    } catch (e) {
      console.error("Microphone access denied:", e);
    }
  };

  const stopRecording = () => {
    mediaRecorder.current?.stop();
    setRecording(false);
  };

  return (
    <button
      onClick={recording ? stopRecording : startRecording}
      disabled={processing}
      title={recording ? "Stop recording" : "Start voice input"}
      style={{
        background: "none",
        border: "none",
        cursor: "pointer",
        padding: "4px 8px",
        fontSize: "18px",
        color: recording ? "#f85149" : "#8b949e",
      }}
    >
      {processing ? "⏳" : recording ? "🔴" : "🎤"}
    </button>
  );
}
