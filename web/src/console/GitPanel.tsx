import { useState, useEffect } from "react";
import { useTranslation } from '../i18n';
import { invoke } from "@tauri-apps/api/core";

interface GitCommit {
  hash: string;
  author: string;
  message: string;
  time: string;
}

interface GitInfoData {
  repo_path: string;
  branch: string;
  uncommitted_changes: number;
  recent_commits: GitCommit[];
  is_git_repo: boolean;
  error: string | null;
}

const cardStyle: React.CSSProperties = {
  background: "#161b22",
  borderRadius: "8px",
  padding: "16px",
  border: "1px solid #30363d",
};

export default function GitPanel() {
  const { t } = useTranslation();
  const [info, setInfo] = useState<GitInfoData | null>(null);
  const [loading, setLoading] = useState(true);

  const fetchInfo = async () => {
    setLoading(true);
    try {
      const result = await invoke<GitInfoData>("git_info");
      setInfo(result);
    } catch (e) {
      setInfo({
        repo_path: "",
        branch: "",
        uncommitted_changes: 0,
        recent_commits: [],
        is_git_repo: false,
        error: String(e),
      });
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchInfo();
  }, []);

  if (loading) {
    return (
      <div>
        <h2 style={{ color: "#e6edf3", marginBottom: "16px" }}>{t('console.git.title')}</h2>
        <div style={{ color: "#8b949e" }}>{t('console.git.loading')}</div>
      </div>
    );
  }

  if (!info) {
    return (
      <div>
        <h2 style={{ color: "#e6edf3", marginBottom: "16px" }}>{t('console.git.title')}</h2>
        <div style={{ color: "#8b949e" }}>{t('console.git.no_data')}</div>
      </div>
    );
  }

  const truncateHash = (hash: string) => hash.slice(0, 7);

  return (
    <div>
      <h2 style={{ color: "#e6edf3", marginBottom: "16px" }}>{t('console.git.title')}</h2>

      {!info.is_git_repo ? (
        <div style={cardStyle}>
          <div style={{ color: "#f0883e", fontSize: "14px", fontWeight: 600, marginBottom: "8px" }}>
            {t('console.git.not_repo_title')}
          </div>
          <div style={{ color: "#8b949e", fontSize: "13px" }}>
            {t('console.git.not_repo_desc')}
          </div>
          <div style={{ color: "#8b949e", fontSize: "12px", marginTop: "8px" }}>
            {t('console.git.current_dir')}: {info.repo_path || t('console.git.unknown')}
          </div>
        </div>
      ) : (
        <>
          <div style={{ display: "grid", gridTemplateColumns: "repeat(2, 1fr)", gap: "12px", marginBottom: "16px" }}>
            <div style={cardStyle}>
              <div style={{ color: "#8b949e", fontSize: "13px" }}>{t('console.git.repo_path')}</div>
              <div style={{ color: "#e6edf3", fontSize: "13px", marginTop: "8px", wordBreak: "break-all" }}>
                {info.repo_path}
              </div>
            </div>
            <div style={cardStyle}>
              <div style={{ color: "#8b949e", fontSize: "13px" }}>{t('console.git.branch')}</div>
              <div style={{ color: "#58a6ff", fontSize: "18px", fontWeight: "bold", marginTop: "8px" }}>
                {info.branch || t('console.git.unknown')}
              </div>
            </div>
            <div style={cardStyle}>
              <div style={{ color: "#8b949e", fontSize: "13px" }}>{t('console.git.uncommitted')}</div>
              <div style={{ color: info.uncommitted_changes > 0 ? "#d29922" : "#3fb950", fontSize: "18px", fontWeight: "bold", marginTop: "8px" }}>
                {info.uncommitted_changes}
              </div>
            </div>
            <div style={cardStyle}>
              <div style={{ color: "#8b949e", fontSize: "13px" }}>{t('console.git.actions')}</div>
              <div style={{ marginTop: "8px", display: "flex", gap: "8px" }}>
                <button
                  onClick={fetchInfo}
                  style={{
                    background: "#21262d",
                    color: "#c9d1d9",
                    border: "1px solid #30363d",
                    borderRadius: "6px",
                    padding: "4px 12px",
                    fontSize: "12px",
                    cursor: "pointer",
                  }}
                >
                  {t('console.git.refresh')}
                </button>
                <button
                  onClick={() => invoke("git_info").catch(() => {})}
                  style={{
                    background: "#21262d",
                    color: "#c9d1d9",
                    border: "1px solid #30363d",
                    borderRadius: "6px",
                    padding: "4px 12px",
                    fontSize: "12px",
                    cursor: "pointer",
                  }}
                >
                  {t('console.git.open_repo')}
                </button>
              </div>
            </div>
          </div>

          <h3 style={{ color: "#e6edf3", fontSize: "15px", margin: "0 0 12px 0" }}>
            {t('console.git.recent_commits')}
          </h3>
          <div style={{ ...cardStyle, padding: "8px 16px" }}>
            {info.recent_commits.length === 0 ? (
              <div style={{ color: "#8b949e", fontSize: "13px", padding: "8px 0" }}>
                {t('console.git.no_commits')}
              </div>
            ) : (
              <table style={{ width: "100%", borderCollapse: "collapse" }}>
                <thead>
                  <tr style={{ color: "#8b949e", fontSize: "12px", textAlign: "left" }}>
                    <th style={{ padding: "8px 4px", borderBottom: "1px solid #30363d" }}>{t('console.git.hash')}</th>
                    <th style={{ padding: "8px 4px", borderBottom: "1px solid #30363d" }}>{t('console.git.message')}</th>
                    <th style={{ padding: "8px 4px", borderBottom: "1px solid #30363d" }}>{t('console.git.author')}</th>
                    <th style={{ padding: "8px 4px", borderBottom: "1px solid #30363d" }}>{t('console.git.time')}</th>
                  </tr>
                </thead>
                <tbody>
                  {info.recent_commits.map((commit) => (
                    <tr key={commit.hash} style={{ color: "#c9d1d9", fontSize: "13px" }}>
                      <td style={{ padding: "8px 4px", borderBottom: "1px solid #21262d", fontFamily: "monospace", color: "#58a6ff" }}>
                        {truncateHash(commit.hash)}
                      </td>
                      <td style={{ padding: "8px 4px", borderBottom: "1px solid #21262d" }}>{commit.message}</td>
                      <td style={{ padding: "8px 4px", borderBottom: "1px solid #21262d", color: "#8b949e" }}>{commit.author}</td>
                      <td style={{ padding: "8px 4px", borderBottom: "1px solid #21262d", color: "#8b949e" }}>{commit.time}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            )}
          </div>
        </>
      )}
    </div>
  );
}
