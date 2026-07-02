<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from "vue";
import { useI18n } from "vue-i18n";
import { Component, Package, Sun, Grid2x2Check, LoaderCircle, ChevronLeft, ChevronRight } from "@lucide/vue";
import { getCache, setCache } from "../../utils/cache";

const { t } = useI18n();

const activeTab = ref("mods");

const tabs = [
  { id: "mods", label: "模组", icon: Component },
  { id: "modpack", label: "整合包", icon: Package },
  { id: "shader", label: "光影包", icon: Sun },
  { id: "resourcepack", label: "资源包", icon: Grid2x2Check },
];

interface ModrinthProject {
  project_id: string;
  title: string;
  description: string;
  icon_url: string;
  slug: string;
  project_type: string;
  downloads: number;
  date_modified: string;
  latest_version: string;
  versions: string[];
  game_versions: string[];
  loaders: string[];
}

interface ModrinthSearchHit {
  project_id: string;
  title: string;
  description: string;
  icon_url: string;
  slug: string;
  project_type: string;
  downloads: number;
  date_modified: string;
  latest_version: string;
  versions: string[];
  game_versions: string[];
  loaders: string[];
}

interface ModrinthSearchResult {
  hits: ModrinthSearchHit[];
  total_hits: number;
}

const loading = ref(false);
const projects = ref<ModrinthProject[]>([]);
const currentPage = ref(1);
const totalHits = ref(0);
const pageSize = ref(10);

const totalPages = computed(() => Math.max(1, Math.ceil(totalHits.value / pageSize.value)));

const areaRef = ref<HTMLElement | null>(null);
let resizeObserver: ResizeObserver | null = null;
let resizeTimer: ReturnType<typeof setTimeout> | null = null;
let prevPageSize = 0;

function updatePageSize() {
  if (!areaRef.value) return;
  const available = areaRef.value.clientHeight - 42;
  const cardUnit = 100 + 12;
  const newSize = Math.max(1, Math.floor(available / cardUnit));
  if (newSize !== prevPageSize) {
    prevPageSize = newSize;
    pageSize.value = newSize;
    if (resizeTimer) clearTimeout(resizeTimer);
    resizeTimer = setTimeout(() => {
      currentPage.value = 1;
      fetchProjects();
    }, 400);
  }
}

onMounted(() => {
  updatePageSize();
  resizeObserver = new ResizeObserver(updatePageSize);
  if (areaRef.value) resizeObserver.observe(areaRef.value);
});

onUnmounted(() => {
  resizeObserver?.disconnect();
});

const projectTypeMap: Record<string, string> = {
  mods: "mod",
  modpack: "modpack",
  shader: "shader",
  resourcepack: "resourcepack",
};

async function fetchProjects() {
  const type = projectTypeMap[activeTab.value];
  const page = currentPage.value;
  const limit = pageSize.value;
  const offset = (page - 1) * limit;
  const cacheKey = "modrinth_" + type + "_p" + page + "_l" + limit;
  const cached = getCache<{ projects: ModrinthProject[]; total: number }>(cacheKey);
  if (cached) {
    projects.value = cached.projects;
    totalHits.value = cached.total;
    return;
  }
  loading.value = true;
  try {
    const res = await fetch(
      `https://api.modrinth.com/v2/search?facets=[["project_type:${type}"]]&limit=${limit}&offset=${offset}&index=downloads`
    );
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    const data: ModrinthSearchResult = await res.json();
    totalHits.value = data.total_hits;
    projects.value = data.hits.map((h) => ({
      project_id: h.project_id,
      title: h.title,
      description: h.description,
      icon_url: h.icon_url,
      slug: h.slug,
      project_type: h.project_type,
      downloads: h.downloads,
      date_modified: h.date_modified,
      latest_version: h.latest_version,
      versions: h.versions || [],
      game_versions: h.game_versions || [],
      loaders: h.loaders || [],
    }));
    setCache(cacheKey, { projects: projects.value, total: totalHits.value });
  } catch {
    projects.value = [];
    totalHits.value = 0;
  } finally {
    loading.value = false;
  }
}

watch(activeTab, () => {
  currentPage.value = 1;
  fetchProjects();
});
onMounted(fetchProjects);

function formatDate(iso: string): string {
  try {
    const d = new Date(iso);
    return d.toLocaleDateString(undefined, {
      year: "numeric", month: "short", day: "numeric",
    });
  } catch {
    return iso;
  }
}
</script>

<template>
  <div class="resources-page">
    <div class="resources-header">
      <span class="resources-header-sub">{{ t("app.mainwindow.resourcescenter.title") }}</span>
      <div class="resources-header-row">
        <span class="resources-header-main">{{ t("app.mainwindow.resourcescenter.mainTitle") }}</span>
        <div class="resources-tabs">
          <button
            v-for="tab in tabs"
            :key="tab.id"
            class="resources-tab"
            :class="{ active: activeTab === tab.id }"
            @click="activeTab = tab.id"
          >
            <component :is="tab.icon" :size="16" />
            <span>{{ tab.label }}</span>
          </button>
        </div>
      </div>
    </div>
    <div ref="areaRef" class="resources-area">
      <div v-if="loading" class="resources-loading">
        <LoaderCircle :size="20" class="spinner" />
        <span>正在获取资源列表...</span>
      </div>
      <div v-else-if="projects.length === 0" class="resources-empty">
        暂无资源
      </div>
      <div v-else class="project-list">
          <a
            v-for="project in projects"
            :key="project.project_id"
            :href="`https://modrinth.com/${project.project_type}/${project.slug}`"
            target="_blank"
            class="project-card-hero"
          >
            <img
              :src="project.icon_url || ''"
              :alt="project.title"
              class="project-card-hero-img"
            />
            <div class="project-card-hero-overlay"></div>
            <div class="project-card-hero-content">
              <div class="project-card-row">
                <img :src="project.icon_url" class="project-card-mc-icon" />
                <div class="project-card-info">
                  <span class="project-card-title">{{ project.title }}</span>
                  <span class="project-card-meta">
                    更新于 {{ formatDate(project.date_modified) }}
                  </span>
                  <div class="project-card-tags">
                    <span v-if="project.loaders.length" class="project-tag">
                      {{ project.loaders.slice(0, 3).join(", ") }}
                    </span>
                    <span v-if="project.game_versions.length" class="project-tag">
                      {{ project.game_versions.slice(0, 3).join(", ") }}
                    </span>
                  </div>
                </div>
              </div>
            </div>
          </a>
      </div>
      <div v-if="totalPages > 1" class="pagination">
        <button class="page-btn" :disabled="currentPage <= 1" @click="currentPage--; fetchProjects()">
          <ChevronLeft :size="16" />
        </button>
        <span class="page-info">{{ currentPage }} / {{ totalPages }}</span>
        <button class="page-btn" :disabled="currentPage >= totalPages" @click="currentPage++; fetchProjects()">
          <ChevronRight :size="16" />
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.resources-page {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  padding: 24px 28px;
  overflow-y: auto;
}

.resources-header {
  display: flex;
  flex-direction: column;
  gap: 2px;
  flex-shrink: 0;
}

.resources-header-sub {
  font-size: 11px;
  color: var(--title-color);
  opacity: 0.45;
  line-height: 1;
}

.resources-header-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.resources-header-main {
  font-size: 18px;
  font-weight: 600;
  color: var(--title-color);
  line-height: 1.2;
}

.resources-tabs {
  display: flex;
  gap: 4px;
}

.resources-tab {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 16px;
  border: none;
  border-radius: 100px;
  background: transparent;
  color: var(--title-color);
  opacity: 0.5;
  font-size: 13px;
  font-family: inherit;
  cursor: pointer;
  transition: background 0.15s, opacity 0.15s;
}

.resources-tab:hover {
  background: rgba(128, 128, 128, 0.1);
  opacity: 0.8;
}

.resources-tab.active {
  background: rgba(128, 128, 128, 0.12);
  opacity: 1;
}

.resources-area {
  flex: 1;
  margin-top: 20px;
  min-height: 0;
  display: flex;
  flex-direction: column;
}

.resources-loading {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 10px;
  height: 100%;
  color: var(--title-color);
  opacity: 0.6;
  font-size: 13px;
}

.resources-empty {
  text-align: center;
  color: var(--title-color);
  opacity: 0.4;
  font-size: 13px;
  margin-top: 40px;
}

.spinner {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.project-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
  flex: 1;
  overflow-y: auto;
  min-height: 0;
}

.project-card-hero {
  position: relative;
  width: 100%;
  height: 100px;
  border-radius: 12px;
  overflow: hidden;
  text-decoration: none;
  color: inherit;
  cursor: pointer;
  display: block;
  flex-shrink: 0;
}

.project-card-hero-img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  display: block;
  transition: transform 0.3s ease;
}

.project-card-hero:hover .project-card-hero-img {
  transform: scale(1.05);
}

.project-card-hero-overlay {
  position: absolute;
  inset: 0;
  background: linear-gradient(transparent 30%, rgba(0, 0, 0, 0.8));
  pointer-events: none;
}

.project-card-hero-content {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  padding: 16px;
}

.project-card-row {
  display: flex;
  align-items: flex-start;
  gap: 12px;
}

.project-card-mc-icon {
  width: 40px;
  height: 40px;
  border-radius: 8px;
  flex-shrink: 0;
}

.project-card-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}

.project-card-title {
  font-size: 16px;
  font-weight: 700;
  color: #fff;
  line-height: 1.2;
}

.project-card-meta {
  font-size: 11px;
  color: rgba(255, 255, 255, 0.6);
  line-height: 1;
}

.project-card-tags {
  display: flex;
  gap: 6px;
  margin-top: 4px;
  flex-wrap: wrap;
}

.project-tag {
  font-size: 11px;
  padding: 2px 8px;
  border-radius: 4px;
  background: rgba(255, 255, 255, 0.15);
  color: rgba(255, 255, 255, 0.85);
}

.pagination {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 12px;
  padding: 6px 0 16px;
  flex-shrink: 0;
}

.page-info {
  font-size: 13px;
  color: var(--title-color);
  opacity: 0.6;
  min-width: 60px;
  text-align: center;
}

.page-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border: 1px solid rgba(128, 128, 128, 0.2);
  border-radius: 6px;
  background: var(--panel-bg);
  color: var(--title-color);
  font-size: 13px;
  font-family: inherit;
  cursor: pointer;
  transition: background 0.15s, border-color 0.15s;
}

.page-btn:hover:not(:disabled) {
  background: rgba(128, 128, 128, 0.1);
  border-color: rgba(128, 128, 128, 0.3);
}

.page-btn:disabled {
  opacity: 0.25;
  cursor: not-allowed;
}
</style>
