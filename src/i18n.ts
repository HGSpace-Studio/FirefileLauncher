import { createI18n } from "vue-i18n";
import zh_cn from "./lang/zh_cn.json";
import en_us from "./lang/en_us.json";
import ko_kr from "./lang/ko_kr.json";
import fr from "./lang/fr.json";
import ru from "./lang/ru.json";
import vi from "./lang/vi.json";

const messages = {
  zh_cn,
  en_us,
  ko_kr,
  fr,
  ru,
  vi,
};

export function getSystemLocale(): string {
  const langs = navigator.languages || [navigator.language];
  for (const lang of langs) {
    const key = lang.toLowerCase().replace("-", "_");
    if (key.startsWith("zh")) return "zh_cn";
    if (key.startsWith("ko")) return "ko_kr";
    if (key.startsWith("fr")) return "fr";
    if (key.startsWith("ru")) return "ru";
    if (key.startsWith("vi")) return "vi";
    if (key.startsWith("en")) return "en_us";
  }
  return "en_us";
}

export default createI18n({
  locale: getSystemLocale(),
  fallbackLocale: "en_us",
  messages,
});
