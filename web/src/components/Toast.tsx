import { useEffect, useCallback, useState } from "react";

export type ToastType = "success" | "error" | "info" | "warning";

export interface ToastData {
  id: number;
  type: ToastType;
  message: string;
}

interface ToastItemProps {
  toast: ToastData;
  onRemove: (id: number) => void;
}

const icons: Record<ToastType, string> = {
  success: "✓",
  error: "✕",
  info: "ℹ",
  warning: "⚠",
};

export function ToastItem({ toast, onRemove }: ToastItemProps) {
  const [exiting, setExiting] = useState(false);

  useEffect(() => {
    const timer = setTimeout(() => {
      setExiting(true);
      setTimeout(() => onRemove(toast.id), 200);
    }, 3000);
    return () => clearTimeout(timer);
  }, [toast.id, onRemove]);

  const handleClose = useCallback(() => {
    setExiting(true);
    setTimeout(() => onRemove(toast.id), 200);
  }, [toast.id, onRemove]);

  return (
    <div className={`toast toast-${toast.type}${exiting ? ' toast-exit' : ''}`}>
      <span className="toast-icon">{icons[toast.type]}</span>
      <span className="toast-message">{toast.message}</span>
      <button className="toast-close" onClick={handleClose}>×</button>
    </div>
  );
}