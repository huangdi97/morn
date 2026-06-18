import React, { createContext, useContext, useState, useCallback } from 'react';
import zh from './zh.json';
import en from './en.json';

type Locale = 'zh' | 'en';
type Messages = Record<string, string>;

const messages: Record<Locale, Messages> = { zh, en };

const STORAGE_KEY = 'morn-locale';

function getInitialLocale(): Locale {
  const stored = localStorage.getItem(STORAGE_KEY);
  if (stored === 'zh' || stored === 'en') return stored;
  return navigator.language.startsWith('zh') ? 'zh' : 'en';
}

type LocaleContextType = {
  locale: Locale;
  setLocale: (l: Locale) => void;
  t: (key: string, params?: Record<string, string | number>) => string;
};

const LocaleContext = createContext<LocaleContextType>({
  locale: getInitialLocale(),
  setLocale: () => {},
  t: (key: string) => key,
});

export function LocaleProvider({ children }: { children: React.ReactNode }) {
  const [locale, setLocaleState] = useState<Locale>(getInitialLocale);

  const setLocale = useCallback((l: Locale) => {
    localStorage.setItem(STORAGE_KEY, l);
    setLocaleState(l);
  }, []);

  const t = useCallback((key: string, params?: Record<string, string | number>): string => {
    let value = messages[locale]?.[key];
    if (value === undefined) {
      value = messages['en']?.[key];
    }
    if (value === undefined) return key;
    if (params) {
      for (const [k, v] of Object.entries(params)) {
        value = value.replace(`{${k}}`, String(v));
      }
    }
    return value;
  }, [locale]);

  return (
    <LocaleContext.Provider value={{ locale, setLocale, t }}>
      {children}
    </LocaleContext.Provider>
  );
}

export function useTranslation() {
  const ctx = useContext(LocaleContext);
  return { t: ctx.t, locale: ctx.locale, setLocale: ctx.setLocale };
}
