<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useI18n } from "vue-i18n";
import { LoaderCircle, ExternalLink } from "@lucide/vue";

const { t } = useI18n();

interface NewsImage {
  title: string;
  url: string;
  dimensions?: { width: number; height: number };
}

interface NewsEntry {
  title: string;
  tag: string;
  category: string;
  date: string;
  text: string;
  playPageImage: NewsImage;
  newsPageImage: NewsImage;
  readMoreLink: string;
  newsType: string[];
  id: string;
}

interface NewsResponse {
  version: number;
  entries: NewsEntry[];
}

const baseUrl = "https://launchercontent.mojang.com";
const newsUrl = `${baseUrl}/v2/news.json`;

const loading = ref(true);
const error = ref("");
const javaNews = ref<NewsEntry[]>([]);

function formatDate(iso: string): string {
  try {
    const d = new Date(iso);
    return d.toLocaleDateString(undefined, {
      year: "numeric",
      month: "long",
      day: "numeric",
    });
  } catch {
    return iso;
  }
}

onMounted(async () => {
  try {
    const res = await fetch(newsUrl);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    const data: NewsResponse = await res.json();
    javaNews.value = data.entries.filter(
      (e) => e.category === "Minecraft: Java Edition"
    );
    if (javaNews.value.length === 0) {
      javaNews.value = data.entries.slice(0, 10);
    }
  } catch (e: any) {
    error.value = e.message || String(e);
  } finally {
    loading.value = false;
  }
});
</script>

<template>
  <div class="home-page">
    <div class="home-header">
      <span class="home-header-sub">{{ t("app.mainwindow.home.title") }}</span>
      <span class="home-header-main">Minecraft News</span>
    </div>
    <div class="news-area">
      <div v-if="loading" class="news-loading">
        <LoaderCircle :size="20" class="spinner" />
        <span>{{ t("app.mainwindow.home.fetching") }}</span>
      </div>
      <div v-else-if="error" class="news-error">{{ t("app.mainwindow.home.loadError") }} {{ error }}</div>
      <div v-else class="news-list">
        <div class="hero-row">
          <a
            v-for="entry in javaNews.slice(0, 3)"
            :key="entry.id"
            :href="entry.readMoreLink"
            target="_blank"
            class="news-card-hero"
          >
            <img
              :src="`${baseUrl}${entry.newsPageImage?.url || entry.playPageImage?.url}`"
              :alt="entry.title"
              class="news-card-hero-img"
            />
            <div class="news-card-hero-overlay"></div>
            <div class="news-card-hero-content">
              <span class="hero-date">{{ formatDate(entry.date) }}</span>
              <span class="hero-title">{{ entry.title }}</span>
              <p class="hero-text">{{ entry.text }}</p>
            </div>
          </a>
        </div>
        <a
          v-for="entry in javaNews.slice(3)"
          :key="entry.id"
          :href="entry.readMoreLink"
          target="_blank"
          class="news-card"
        >
          <img
            v-if="entry.newsPageImage?.url"
            :src="`${baseUrl}${entry.newsPageImage.url}`"
            :alt="entry.title"
            class="news-card-img"
          />
          <div class="news-card-body">
            <span class="news-card-date">{{ formatDate(entry.date) }}</span>
            <span class="news-card-title">{{ entry.title }}</span>
            <p class="news-card-text">{{ entry.text }}</p>
            <span class="news-card-link">
              {{ t("app.mainwindow.home.readMore") }}
              <ExternalLink :size="12" />
            </span>
          </div>
        </a>
      </div>
    </div>
  </div>
</template>

<style scoped>
.home-page {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  padding: 24px 28px;
  overflow-y: auto;
}

.home-header {
  display: flex;
  flex-direction: column;
  gap: 2px;
  flex-shrink: 0;
}

.home-header-sub {
  font-size: 11px;
  color: var(--title-color);
  opacity: 0.45;
  line-height: 1;
}

.home-header-main {
  font-size: 18px;
  font-weight: 600;
  color: var(--title-color);
  line-height: 1.2;
}

.news-area {
  flex: 1;
  margin-top: 20px;
  min-height: 0;
}

.news-loading {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 10px;
  height: 100%;
  color: var(--title-color);
  opacity: 0.6;
  font-size: 13px;
}

.news-error {
  text-align: center;
  color: #e74c3c;
  font-size: 13px;
  opacity: 0.8;
  margin-top: 40px;
}

.spinner {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.news-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.hero-row {
  display: flex;
  gap: 12px;
}

.news-card-hero {
  position: relative;
  flex: 1;
  height: 440px;
  border-radius: 12px;
  overflow: hidden;
  text-decoration: none;
  color: inherit;
  cursor: pointer;
  display: block;
  min-width: 0;
}

.news-card-hero-img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  display: block;
  transition: transform 0.3s ease;
}

.news-card-hero:hover .news-card-hero-img {
  transform: scale(1.05);
}

.news-card-hero-overlay {
  position: absolute;
  inset: 0;
  background: linear-gradient(transparent 40%, rgba(0, 0, 0, 0.75));
  pointer-events: none;
}

.news-card-hero-content {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  padding: 20px;
  display: flex;
  flex-direction: column;
  gap: 3px;
}

.hero-date {
  font-size: 11px;
  color: rgba(255, 255, 255, 0.6);
  line-height: 1;
}

.hero-title {
  font-size: 18px;
  font-weight: 700;
  color: #fff;
  line-height: 1.2;
}

.hero-text {
  font-size: 13px;
  color: rgba(255, 255, 255, 0.75);
  line-height: 1.3;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
  margin: 0;
}

.news-card {
  display: flex;
  gap: 16px;
  padding: 14px;
  border-radius: 12px;
  background: var(--panel-bg);
  text-decoration: none;
  color: inherit;
  transition: background 0.15s;
  cursor: pointer;
}

.news-card:hover {
  background: rgba(128, 128, 128, 0.12);
}

.news-card-img {
  width: 180px;
  height: 100px;
  border-radius: 8px;
  object-fit: cover;
  flex-shrink: 0;
}

.news-card-body {
  display: flex;
  flex-direction: column;
  gap: 4px;
  min-width: 0;
  flex: 1;
}

.news-card-date {
  font-size: 11px;
  color: var(--title-color);
  opacity: 0.45;
  line-height: 1;
}

.news-card-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--title-color);
  line-height: 1.3;
}

.news-card-text {
  font-size: 12px;
  color: var(--title-color);
  opacity: 0.7;
  line-height: 1.4;
  display: -webkit-box;
  -webkit-line-clamp: 3;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.news-card-link {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  color: var(--sidebar-active-color);
  margin-top: auto;
}
</style>
