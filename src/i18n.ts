import { createI18n } from "vue-i18n";
import zh_cn from "./lang/zh_cn.json";
import en_us from "./lang/en_us.json";

const messages = {
  zh_cn,
  en_us,
};

export function getSystemLocale(): string {
  const langs = navigator.languages || [navigator.language];
  for (const lang of langs) {
    const key = lang.toLowerCase().replace("-", "_");
    if (key.startsWith("zh")) return "zh_cn";
    if (key.startsWith("en")) return "en_us";
  }
  return "en_us";
}

export default createI18n({
  locale: getSystemLocale(),
  fallbackLocale: "en_us",
  messages,
});
