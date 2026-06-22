import { useState, useEffect } from "react";
import "./design.css";

interface CssVar {
  name: string;
  value: string;
}

const COLOR_VARS = [
  "bg-page", "bg-panel", "bg-surface", "bg-hover",
  "text-primary", "text-secondary", "text-tertiary", "text-muted",
  "accent", "accent-hover", "accent-active",
  "success", "danger", "warning",
  "border-default", "border-subtle",
  "user-bubble", "assistant-bubble",
];

const SPACE_VARS = ["space-xs", "space-sm", "space-md", "space-lg", "space-xl"];
const RADIUS_VARS = ["radius-sm", "radius-md", "radius-lg", "radius-xl"];

function useCssVars(names: string[]): CssVar[] {
  const [vars, setVars] = useState<CssVar[]>([]);

  useEffect(() => {
    const style = getComputedStyle(document.documentElement);
    setVars(names.map(name => ({
      name: `--${name}`,
      value: style.getPropertyValue(`--${name}`).trim(),
    })));
  }, []);

  return vars;
}

function Swatch({ value }: { value: string }) {
  return (
    <span
      className="ds-swatch"
      style={{ background: value }}
    />
  );
}

function VarRow({ name, value, showSwatch }: { name: string; value: string; showSwatch?: boolean }) {
  return (
    <div className="ds-var-row">
      <code className="ds-var-name">{name}</code>
      {showSwatch && <Swatch value={value} />}
      <span className="ds-var-value">{value}</span>
    </div>
  );
}

function Section({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <section className="ds-section">
      <h2 className="ds-section-title">{title}</h2>
      <div className="ds-section-body">{children}</div>
    </section>
  );
}

function ColorPalette({ vars }: { vars: CssVar[] }) {
  return (
    <div className="ds-palette">
      {vars.map(v => <VarRow key={v.name} name={v.name} value={v.value} showSwatch />)}
    </div>
  );
}

function TokenList({ vars }: { vars: CssVar[] }) {
  return (
    <div className="ds-token-list">
      {vars.map(v => <VarRow key={v.name} name={v.name} value={v.value} />)}
    </div>
  );
}

function DemoButton({ variant = "primary", children }: { variant?: "primary" | "secondary" | "danger" | "ghost"; children: React.ReactNode }) {
  const cls = `ds-btn ds-btn-${variant}`;
  return <button className={cls}>{children}</button>;
}

export function DesignSystem() {
  const colors = useCssVars(COLOR_VARS);
  const spaces = useCssVars(SPACE_VARS);
  const radii = useCssVars(RADIUS_VARS);

  return (
    <div className="ds-page">
      <header className="ds-header">
        <h1 className="ds-title">Morn 设计系统</h1>
        <p className="ds-subtitle">设计令牌与组件样式参考</p>
      </header>

      <Section title="设计原则">
        <div className="ds-principles">
          <div className="ds-principle">
            <span className="ds-principle-num">1</span>
            <div>
              <strong>内容为王</strong>
              <p>UI 不抢内容注意力，白色背景让信息突出</p>
            </div>
          </div>
          <div className="ds-principle">
            <span className="ds-principle-num">2</span>
            <div>
              <strong>一致的节奏</strong>
              <p>8px 基准网格，所有尺寸是 4 的倍数</p>
            </div>
          </div>
          <div className="ds-principle">
            <span className="ds-principle-num">3</span>
            <div>
              <strong>清晰的层级</strong>
              <p>卡片阴影 + 间距区分优先级</p>
            </div>
          </div>
          <div className="ds-principle">
            <span className="ds-principle-num">4</span>
            <div>
              <strong>克制的配色</strong>
              <p>青色只用于交互元素（链接、按钮、焦点），不用于装饰</p>
            </div>
          </div>
        </div>
      </Section>

      <Section title="色彩">
        <ColorPalette vars={colors} />
      </Section>

      <Section title="间距">
        <TokenList vars={spaces} />
      </Section>

      <Section title="圆角">
        <TokenList vars={radii} />
      </Section>

      <Section title="组件展示">
        <h3 className="ds-subheading">Button</h3>
        <div className="ds-component-row">
          <DemoButton variant="primary">Primary</DemoButton>
          <DemoButton variant="secondary">Secondary</DemoButton>
          <DemoButton variant="danger">Danger</DemoButton>
          <DemoButton variant="ghost">Ghost</DemoButton>
        </div>

        <h3 className="ds-subheading">Input</h3>
        <div className="ds-component-col">
          <input className="ds-input" placeholder="默认输入框" />
          <input className="ds-input" placeholder="聚焦状态" autoFocus readOnly />
          <input className="ds-input" placeholder="禁用状态" disabled />
          <input className="ds-input ds-input-error" placeholder="错误状态" defaultValue="错误内容" />
        </div>

        <h3 className="ds-subheading">Card</h3>
        <div className="ds-component-row">
          <div className="ds-card">
            <div className="ds-card-title">卡片标题</div>
            <div className="ds-card-body">这是卡片内容示例文本，展示基础卡片组件的样式。</div>
          </div>
          <div className="ds-card">
            <div className="ds-card-title">卡片标题</div>
            <div className="ds-card-body">另一张卡片展示网格布局效果。</div>
          </div>
        </div>

        <h3 className="ds-subheading">Badge</h3>
        <div className="ds-component-row">
          <span className="ds-badge ds-badge-accent">进行中</span>
          <span className="ds-badge ds-badge-success">已完成</span>
          <span className="ds-badge ds-badge-danger">失败</span>
          <span className="ds-badge ds-badge-warning">待处理</span>
        </div>

        <h3 className="ds-subheading">Toast</h3>
        <div className="ds-component-col">
          <div className="ds-toast-example ds-toast-success">
            <span className="ds-toast-icon">✓</span>
            <span>操作成功</span>
          </div>
          <div className="ds-toast-example ds-toast-error">
            <span className="ds-toast-icon">✕</span>
            <span>操作失败</span>
          </div>
          <div className="ds-toast-example ds-toast-info">
            <span className="ds-toast-icon">ℹ</span>
            <span>提示信息</span>
          </div>
          <div className="ds-toast-example ds-toast-warning">
            <span className="ds-toast-icon">⚠</span>
            <span>警告信息</span>
          </div>
        </div>
      </Section>
    </div>
  );
}
