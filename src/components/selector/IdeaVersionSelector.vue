<script setup>
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

const emit = defineEmits(["select-version"]);

const versions = ref([]);
const loading = ref(false);
const selectedVersion = ref(null);
const manualPath = ref("");
const useManual = ref(false);

async function loadVersions() {
  loading.value = true;
  try {
    const [foundVersions, savedPath] = await Promise.all([
      invoke("find_idea_dirs"),
      invoke("get_saved_idea_version")
    ]);
    
    versions.value = foundVersions;
    
    if (versions.value.length > 0) {
      if (savedPath) {
        const saved = versions.value.find(v => v.workspace_path === savedPath);
        if (saved) {
          selectedVersion.value = saved.workspace_path;
        } else {
          manualPath.value = savedPath;
          useManual.value = true;
          handleManualSubmit();
          return;
        }
      } else {
        selectedVersion.value = versions.value[0].workspace_path;
      }
      handleSelect();
    } else {
      useManual.value = true;
      if (savedPath) {
        manualPath.value = savedPath;
        handleManualSubmit();
      }
    }
  } catch (error) {
    console.error("Failed to find IDEA directories:", error);
    useManual.value = true;
  } finally {
    loading.value = false;
  }
}

async function handleSelect() {
  if (selectedVersion.value) {
    try {
      await invoke("save_idea_version", { workspacePath: selectedVersion.value });
    } catch (e) {
      console.warn("Failed to save preference:", e);
    }
    emit("select-version", selectedVersion.value);
  }
}

async function handleManualSubmit() {
  if (manualPath.value) {
    try {
      await invoke("save_idea_version", { workspacePath: manualPath.value });
    } catch (e) {
      console.warn("Failed to save preference:", e);
    }
    emit("select-version", manualPath.value);
  }
}

async function browseFolder() {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: "选择 IDEA workspace 目录"
    });
    if (selected) {
      manualPath.value = selected;
    }
  } catch (e) {
    console.error("Failed to open folder dialog:", e);
  }
}

function backToAutoSelect() {
  useManual.value = false;
  // 切换回自动选择时，重新触发选择事件
  if (selectedVersion.value) {
    handleSelect();
  }
}

onMounted(() => {
  loadVersions();
});
</script>

<template>
  <el-card class="version-selector" shadow="hover">
    <div class="selector-content">
      <div class="selector-label">
        <el-icon><Setting /></el-icon>
        <span>IDEA 版本</span>
      </div>
      
      <el-skeleton v-if="loading" :rows="1" animated style="width: 200px" />
      
      <div class="input-area" v-if="!useManual && versions.length > 0">
        <el-select 
          v-model="selectedVersion" 
          @change="handleSelect"
          placeholder="选择 IDEA 版本"
          class="version-select"
        >
          <el-option
            v-for="v in versions"
            :key="v.workspace_path"
            :label="v.name"
            :value="v.workspace_path"
          />
        </el-select>
        <el-button @click="useManual = true">
          <el-icon><Edit /></el-icon>
          手动指定
        </el-button>
      </div>

      <div class="input-area" v-else>
        <el-input 
          v-model="manualPath" 
          placeholder="选择或输入 workspace 目录路径"
          @keyup.enter="handleManualSubmit"
          clearable
          class="path-input"
        >
          <template #append>
            <el-button @click="browseFolder">
              <el-icon><FolderOpened /></el-icon>
            </el-button>
          </template>
        </el-input>
        <el-button type="primary" @click="handleManualSubmit">确认</el-button>
        <el-button v-if="versions.length > 0" @click="backToAutoSelect">返回</el-button>
      </div>
    </div>
  </el-card>
</template>

<style scoped>
.version-selector {
  flex: 1;
  min-width: 0;
}

.selector-content {
  display: flex;
  align-items: center;
  gap: 12px;
}

.selector-label {
  display: flex;
  align-items: center;
  gap: 6px;
  font-weight: 500;
  color: #303133;
  white-space: nowrap;
}

.input-area {
  display: flex;
  align-items: center;
  gap: 8px;
  flex: 1;
  min-width: 0;
}

.version-select {
  width: 180px;
}

.path-input {
  flex: 1;
  min-width: 150px;
}

.path-input :deep(.el-input__wrapper) {
  box-shadow: none !important;
  border: 1px solid var(--el-input-border-color);
  border-right: none;
}

.path-input :deep(.el-input__inner) {
  box-shadow: none !important;
}

.path-input :deep(.el-input__wrapper.is-focus) {
  box-shadow: none !important;
  border-color: var(--el-color-primary);
}

.path-input :deep(.el-input-group__append) {
  background-color: var(--el-fill-color-light);
  box-shadow: none !important;
  border: 1px solid var(--el-input-border-color);
  border-left: 1px solid var(--el-input-border-color);
}

.path-input :deep(.el-input-group__append .el-button) {
  box-shadow: none !important;
  border: none !important;
  background-color: transparent;
  padding: 0 15px;
  margin: 0;
}

:deep(.el-input-group__append) {
  padding: 0;
}
</style>
