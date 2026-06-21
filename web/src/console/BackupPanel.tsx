import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useTranslation } from '../i18n';

interface BackupEntry {
  name: string;
  path: string;
  size: string;
}

export function BackupPanel() {
  const { t } = useTranslation();
  const [backupList, setBackupList] = useState<BackupEntry[]>([]);
  const [loading, setLoading] = useState(false);
  const [message, setMessage] = useState<string | null>(null);

  const handleBackup = async () => {
    setLoading(true);
    setMessage(null);
    try {
      const timestamp = new Date().toISOString().slice(0, 19).replace(/[:-]/g, '');
      const path = `morn-backups/morn-${timestamp}.mornbackup`;
      const result = await invoke<string>("create_backup", { path });
      setMessage(result);
      loadBackups();
    } catch (e) {
      setMessage(`Backup failed: ${e}`);
    } finally {
      setLoading(false);
    }
  };

  const handleRestore = async (path: string) => {
    if (!confirm(t('backup.confirm_restore'))) return;
    setLoading(true);
    setMessage(null);
    try {
      const result = await invoke<string>("restore_backup", { path });
      setMessage(result);
      loadBackups();
    } catch (e) {
      setMessage(`Restore failed: ${e}`);
    } finally {
      setLoading(false);
    }
  };

  const loadBackups = async () => {
    try {
      const entries: BackupEntry[] = [];
      setBackupList(entries);
    } catch {
      setBackupList([]);
    }
  };

  useEffect(() => { loadBackups(); }, []);

  return (
    <div className="backup-panel">
      <h2>{t('backup.title')}</h2>
      <button onClick={handleBackup} disabled={loading}>
        {loading ? t('backup.in_progress') : t('backup.create_backup')}
      </button>
      {message && <p className="backup-message">{message}</p>}
      <div className="backup-list">
        <h3>{t('backup.backup_list')}</h3>
        {backupList.length === 0 && <p>{t('backup.no_backups')}</p>}
        {backupList.map(b => (
          <div key={b.name} className="backup-item">
            <span>{b.name}</span>
            <span>{b.size}</span>
            <button onClick={() => handleRestore(b.path)}>{t('backup.restore')}</button>
          </div>
        ))}
      </div>
    </div>
  );
}
