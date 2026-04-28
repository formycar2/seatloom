import { create } from 'zustand';
import { Locale, translations } from '../i18n';

interface LocaleState {
  locale: Locale;
  t: typeof translations.en;
  setLocale: (locale: Locale) => void;
}

export const useLocaleStore = create<LocaleState>((set) => ({
  locale: 'zh',
  t: translations.zh,
  setLocale: (locale) => set({ locale, t: translations[locale] }),
}));
