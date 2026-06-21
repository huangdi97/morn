# Batch 3 — 错误恢复 ErrorBoundary

前端添加专用 ErrorBoundary 组件，按错误类型分类显示用户友好信息。

## 任务列表

### T1: ErrorBoundary 组件

文件：`web/src/components/ErrorBoundary.tsx`（新建）

React Error Boundary 组件：
```tsx
import React from 'react';

interface Props {
  children: React.ReactNode;
  fallback?: React.ReactNode;
  onError?: (error: Error, errorInfo: React.ErrorInfo) => void;
}

interface State {
  hasError: boolean;
  error: Error | null;
}

export class ErrorBoundary extends React.Component<Props, State> {
  state: State = { hasError: false, error: null };

  static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, errorInfo: React.ErrorInfo) {
    console.error('ErrorBoundary caught:', error, errorInfo);
    this.props.onError?.(error, errorInfo);
  }

  render() {
    if (this.state.hasError) {
      if (this.props.fallback) return this.props.fallback;
      return (
        <div className="error-boundary">
          <div className="error-boundary-icon">⚠️</div>
          <h3>Something went wrong</h3>
          <p className="error-boundary-message">
            {this.state.error?.message || 'An unexpected error occurred'}
          </p>
          <button
            className="error-boundary-retry"
            onClick={() => this.setState({ hasError: false, error: null })}
          >
            Try Again
          </button>
        </div>
      );
    }
    return this.props.children;
  }
}
```

### T2: 错误分类显示

在 ErrorBoundary 中按错误类型自定义 fallback：
- 网络错误（fetch failed / network error）→ "Connection lost. Check your network."
- API 错误（rate limit / quota）→ "API limit reached. Try again later."
- 权限错误（permission denied）→ "Permission denied. Check settings."
- 通用错误 → 友好提示 + 重试按钮

### T3: 包裹主视图

文件：`web/src/App.tsx`

用 ErrorBoundary 包裹各主视图：
```tsx
<ErrorBoundary fallback={<ErrorFallback />}>
  {view === "workbench" && renderWorkbench()}
  {view === "studio" && renderStudio()}
  {view === "hub" && ...}
  {view === "console" && renderConsole()}
</ErrorBoundary>
```

可选：也分别包裹每个视图（防止一个视图崩溃影响其他）。

### T4: 重试管线增强

文件：`src-tauri/src/commands/recovery.rs`

当前：`get_last_error` / `retry_last_operation` 基于审计日志。

确认两条命令已正确注册，在 ErrorBoundary 的 "Try Again" 按钮可调用 `retry_last_operation`。

### T5: 样式

文件：`web/src/styles/base.css` 添加 .error-boundary 样式：
```css
.error-boundary {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 60px 20px;
  text-align: center;
}
.error-boundary-icon { font-size: 48px; margin-bottom: 16px; }
.error-boundary-message { color: var(--text-secondary); margin-bottom: 24px; }
.error-boundary-retry { ... }
```

## 验证

- `npm run build` ✅
- `tsc --noEmit` ✅
- 手动模拟组件崩溃 → ErrorBoundary 显示 fallback
- 点击 "Try Again" → 恢复正常渲染
