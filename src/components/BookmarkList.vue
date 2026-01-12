<script setup>
import BookmarkItem from './BookmarkItem.vue';

defineProps({
  bookmarks: {
    type: Array,
    required: true,
  },
});

defineEmits(['delete']);
</script>

<template>
  <div class="bookmark-list-container">
    <div v-if="bookmarks.length === 0" class="empty-state">
      <div class="empty-icon">📂</div>
      <p>暂无书签数据</p>
    </div>
    <div v-else class="bookmark-list">
      <BookmarkItem
        v-for="item in bookmarks"
        :key="item.id"
        :bookmark="item"
        @delete="$emit('delete', $event)"
      />
    </div>
  </div>
</template>

<style scoped>
.bookmark-list-container {
  width: 100%;
}

.empty-state {
  text-align: center;
  padding: 40px;
  color: #888;
  background: #f9f9f9;
  border-radius: 8px;
  border: 1px dashed #ddd;
}

.empty-icon {
  font-size: 48px;
  margin-bottom: 15px;
}

.bookmark-list {
  display: flex;
  flex-direction: column;
  gap: 15px;
}
</style>
