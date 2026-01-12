<script setup>
import { ref, watch, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";

const props = defineProps({
  workspacePath: {
    type: String,
    required: true
  }
});

const emit = defineEmits(["select-file"]);

const files = ref([]);
const loading = ref(false);
const selectedFile = ref(null);

async function loadFiles() {
  if (!props.workspacePath) return;

  loading.value = true;
  try {
    files.value = await invoke("list_workspace_files", { workspacePath: props.workspacePath });
    if (files.value.length > 0) {
      // Try to find bookmarks.xml or similar default
      const defaultFile = files.value.find(f => f.name.toLowerCase().includes("bookmarks"));
      selectedFile.value = defaultFile || files.value[0];
      handleSelect();
    } else {
      selectedFile.value = null;
    }
  } catch (error) {
    console.error("Failed to list workspace files:", error);
  } finally {
    loading.value = false;
  }
}

function handleSelect() {
  if (selectedFile.value) {
    emit("select-file", selectedFile.value.path);
  }
}

watch(() => props.workspacePath, () => {
  loadFiles();
});

onMounted(() => {
  loadFiles();
});
</script>

<template>
  <div class="file-selector">
    <h3>2. 选择配置文件</h3>
    <div v-if="loading" class="loading">正在扫描文件...</div>
    <div v-else-if="files.length === 0" class="empty">该 workspace 目录下没有发现 XML 文件</div>
    <div v-else class="file-list">
      <select v-model="selectedFile" @change="handleSelect">
        <option v-for="f in files" :key="f.path" :value="f">
          {{ f.name }} ({{ f.modified_at }})
        </option>
      </select>
    </div>
  </div>
</template>

<style scoped>
.file-selector {
  background: white;
  padding: 15px;
  border-radius: 8px;
  box-shadow: 0 2px 4px rgba(0,0,0,0.05);
  margin-bottom: 20px;
}

h3 {
  margin-top: 0;
  font-size: 1.1em;
  color: #2c3e50;
  margin-bottom: 15px;
}

select {
  width: 100%;
  padding: 10px;
  border-radius: 4px;
  border: 1px solid #ddd;
}

.loading, .empty {
  color: #666;
  font-style: italic;
}
</style>
