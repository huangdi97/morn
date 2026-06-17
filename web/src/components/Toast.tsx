import { useEffect } from "react";

export interface ToastData {
  id: number;
  type: "success" | "error" | "info";
  message: string;
}

interface ToastItemProps {
  toast: ToastData;
  onRemove: (id: number) => void;
}

export function ToastItem({ toast, onRemove }: ToastItemProps) {
  useEffect(() => {
    const timer = setTimeout(() => onRemove(toast.id), 4000);
    return () => clearTimeout(timer);
  }, [toast.id, onRemove]);

  const icon = {
    success: "\u2713",
    error: "\u2717",
    info: "\u2139",
  }[toast.type];

  return (
    <div className={`toast toast-${toast.type}`}>
      <span className="toast-icon">{icon}</span>
      <span className="toast-message">{toast.message}</span>
    </div>
  );
}