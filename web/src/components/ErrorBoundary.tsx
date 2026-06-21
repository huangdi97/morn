import React from 'react';

function classifyError(error: Error): string {
  const msg = error.message.toLowerCase();
  if (/fetch failed|network error|failed to fetch|networkrequest|i\/o error|connect|timeout/i.test(msg)) {
    return 'network';
  }
  if (/rate limit|quota|too many requests|429/i.test(msg)) {
    return 'api_limit';
  }
  if (/permission denied|forbidden|access denied|403|unauthorized/i.test(msg)) {
    return 'permission';
  }
  return 'generic';
}

const FALLBACK_MESSAGES: Record<string, { icon: string; title: string; message: string }> = {
  network: {
    icon: '📡',
    title: 'Connection Lost',
    message: 'Connection lost. Check your network.',
  },
  api_limit: {
    icon: '⏳',
    title: 'API Limit Reached',
    message: 'API limit reached. Try again later.',
  },
  permission: {
    icon: '🔒',
    title: 'Permission Denied',
    message: 'Permission denied. Check settings.',
  },
  generic: {
    icon: '⚠️',
    title: 'Something went wrong',
    message: 'An unexpected error occurred',
  },
};

interface Props {
  children: React.ReactNode;
  fallback?: React.ReactNode;
  onError?: (error: Error, errorInfo: React.ErrorInfo) => void;
  onRetry?: () => void;
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

  handleRetry = () => {
    this.props.onRetry?.();
    this.setState({ hasError: false, error: null });
  };

  render() {
    if (this.state.hasError) {
      if (this.props.fallback) return this.props.fallback;

      const error = this.state.error!;
      const category = classifyError(error);
      const fb = FALLBACK_MESSAGES[category];

      return (
        <div className="error-boundary">
          <div className="error-boundary-icon">{fb.icon}</div>
          <h3>{fb.title}</h3>
          <p className="error-boundary-message">{fb.message}</p>
          {error.message && category === 'generic' && (
            <p className="error-boundary-detail">{error.message}</p>
          )}
          <button className="error-boundary-retry" onClick={this.handleRetry}>
            Try Again
          </button>
        </div>
      );
    }
    return this.props.children;
  }
}
