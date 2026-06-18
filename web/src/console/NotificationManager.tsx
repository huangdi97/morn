import { useTranslation } from '../i18n';

export default function NotificationManager() {
  const { t } = useTranslation();
  return (
    <div className="notification-manager">
      <h2>{t('console.notifications.title')}</h2>
      <p className="empty-state">No notifications yet.</p>
    </div>
  );
}