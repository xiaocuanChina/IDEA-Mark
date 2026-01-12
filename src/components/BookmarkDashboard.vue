<script setup>
import { ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { Document, FolderOpened } from "@element-plus/icons-vue";
import BookmarkViewer from "./bookmark/BookmarkViewer.vue";
import IdeaVersionSelector from "./selector/IdeaVersionSelector.vue";
import BookmarkStats from "./bookmark/BookmarkStats.vue";
import BackupManager from "./backup/BackupManager.vue";

const currentWorkspacePath = ref("");
const parsedBookmarks = ref([]);
const loading = ref(false);
const errorMessage = ref("");
const activeMenu = ref("bookmarks");

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

function handleMenuSelect(index) {
  activeMenu.value = index;
}
</script>

<template>
  <div class="dashboard">
    <!-- 顶部标题栏 -->
    <div class="header">
      <IdeaVersionSelector @select-version="onVersionSelected" />
      <BookmarkStats 
        v-if="parsedBookmarks.length > 0"
        :stats="bookmarkStats"
        :loading="loading"
        :can-refresh="!!currentWorkspacePath"
        @refresh="refreshBookmarks"
      />
    </div>

    <!-- 主体区域：左侧菜单 + 右侧内容 -->
    <div class="main-container">
      <!-- 左侧菜单 -->
      <div class="sidebar">
        <el-menu
          :default-active="activeMenu"
          class="sidebar-menu"
          @select="handleMenuSelect"
        >
          <el-menu-item index="bookmarks">
            <el-icon><Document /></el-icon>
            <span>查看书签</span>
          </el-menu-item>
          <el-menu-item index="backup">
            <el-icon><FolderOpened /></el-icon>
            <span>书签备份</span>
          </el-menu-item>
        </el-menu>
      </div>

      <!-- 右侧内容区 -->
      <div class="content">
        <el-alert 
          v-if="errorMessage" 
          :title="errorMessage" 
          type="error" 
          show-icon 
          closable
          @close="errorMessage = ''"
          style="margin-bottom: 20px;"
        />

        <!-- 查看书签 -->
        <template v-if="activeMenu === 'bookmarks'">
          <el-card v-if="loading" shadow="never" class="content-card">
            <el-skeleton :rows="5" animated />
          </el-card>
          <el-card v-else shadow="hover" class="content-card">
            <BookmarkViewer :bookmarks="parsedBookmarks" />
          </el-card>
        </template>

        <!-- 书签备份 -->
        <template v-if="activeMenu === 'backup'">
          <el-card shadow="hover" class="content-card">
            <BackupManager 
              v-if="currentWorkspacePath"
              :workspace-path="currentWorkspacePath" 
              @restored="refreshBookmarks"
            />
            <el-empty v-else description="请先选择IDEA版本" />
          </el-card>
        </template>
      </div>
    </div>
  </div>
</template>

<style scoped>
.dashboard {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background: linear-gradient(135deg, #f5f7fa 0%, #e4e8ec 100%);
}

.header {
  display: flex;
  gap: 16px;
  padding: 16px 20px;
  align-items: stretch;
  background: #fff;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.06);
  z-index: 10;
  margin: 12px 12px 0 12px;
  border-radius: 12px;
}

.header > :deep(.version-selector) {
  flex: 1;
  min-width: 0;
}

.header > :deep(.stats-card) {
  flex: 1;
  min-width: 0;
  margin-bottom: 0;
}

.main-container {
  display: flex;
  flex: 1;
  overflow: hidden;
}

.sidebar {
  width: 140px;
  flex-shrink: 0;
  background: #fff;
  box-shadow: 2px 0 8px rgba(0, 0, 0, 0.04);
  padding-top: 16px;
  margin: 12px 0 12px 12px;
  border-radius: 12px;
}

.sidebar-menu {
  height: 100%;
  border-right: none;
  background: transparent;
}

.sidebar-menu :deep(.el-menu-item) {
  margin: 4px 8px;
  border-radius: 8px;
  height: 44px;
  line-height: 44px;
  transition: all 0.2s ease;
}

.sidebar-menu :deep(.el-menu-item:hover) {
  background: #f0f5ff;
}

.sidebar-menu :deep(.el-menu-item.is-active) {
  background: linear-gradient(135deg, #409eff 0%, #66b1ff 100%);
  color: #fff;
}

.sidebar-menu :deep(.el-menu-item.is-active .el-icon) {
  color: #fff;
}

.content {
  flex: 1;
  padding: 24px;
  padding-top: 20px;
  padding-left: 16px;
  overflow: auto;
}

.content-card {
  min-height: 200px;
  border-radius: 12px;
  border: none;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.05);
}

.content-card :deep(.el-card__body) {
  padding: 20px;
}
</style>
