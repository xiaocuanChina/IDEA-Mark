<script setup>
import { ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import BookmarkViewer from "./BookmarkViewer.vue";
import IdeaVersionSelector from "./IdeaVersionSelector.vue";
import BookmarkStats from "./BookmarkStats.vue";
import BackupManager from "./BackupManager.vue";

const currentWorkspacePath = ref("");
const parsedBookmarks = ref([]);
const loading = ref(false);
const errorMessage = ref("");

const bookmarkStats = computed(() => {
  const total = parsedBookmarks.value.length;
  const projects = new Set(parsedBookmarks.value.map(b => b.project_name)).size;
  const withMnemonic = parsedBookmarks.value.filter(b => b.mnemonic).length;
  return { total, projects, withMnemonic };
});

async function loadBookmarksFromWorkspace(workspacePath) {
  if (!workspacePath) return;
  
  loading.value = true;
  errorMessage.value = "";
  
  try {
    parsedBookmarks.value = await invoke("read_bookmarks_from_workspace", { 
      workspacePath: workspacePath 
    });
  } catch (error) {
    console.error("加载书签失败:", error);
    errorMessage.value = "加载书签失败: " + error;
    parsedBookmarks.value = [];
  } finally {
    loading.value = false;
  }
}

function onVersionSelected(workspacePath) {
  currentWorkspacePath.value = workspacePath;
  loadBookmarksFromWorkspace(workspacePath);
}

function refreshBookmarks() {
  if (currentWorkspacePath.value) {
    loadBookmarksFromWorkspace(currentWorkspacePath.value);
  }
}
</script>

<template>
  <div class="dashboard">
    <div class="top-bar">
      <IdeaVersionSelector @select-version="onVersionSelected" />
      
      <BookmarkStats 
        v-if="parsedBookmarks.length > 0"
        :stats="bookmarkStats"
        :loading="loading"
        :can-refresh="!!currentWorkspacePath"
        @refresh="refreshBookmarks"
      />
    </div>

    <el-alert 
      v-if="errorMessage" 
      :title="errorMessage" 
      type="error" 
      show-icon 
      closable
      @close="errorMessage = ''"
      style="margin-bottom: 20px;"
    />

    <!-- 备份与还原面板 -->
    <el-card v-if="currentWorkspacePath" shadow="hover" class="backup-card">
      <BackupManager 
        :workspace-path="currentWorkspacePath" 
        @restored="refreshBookmarks"
      />
    </el-card>

    <el-card v-if="loading" shadow="never" class="loading-card">
      <el-skeleton :rows="5" animated />
    </el-card>

    <el-card v-else shadow="hover" class="viewer-card">
      <BookmarkViewer :bookmarks="parsedBookmarks" />
    </el-card>
  </div>
</template>

<style scoped>
.dashboard {
  width: 100%;
  overflow: hidden;
}

.top-bar {
  display: flex;
  gap: 16px;
  margin-bottom: 20px;
  align-items: stretch;
}

.top-bar > :deep(.version-selector) {
  flex: 0 0 auto;
}

.top-bar > :deep(.stats-card) {
  flex: 1;
  margin-bottom: 0;
}

.loading-card, .viewer-card {
  min-height: 200px;
}

.viewer-card :deep(.el-card__body) {
  padding: 15px;
}

.backup-card {
  margin-bottom: 20px;
}
</style>
