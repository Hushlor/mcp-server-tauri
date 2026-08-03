<template>
   <span v-if="!loading && version" class="version-badge">
      {{ version }}
   </span>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';

const version = ref(''),
      loading = ref(true);

onMounted(async () => {
   try {
      // Fetch latest release from GitHub API
      const response = await fetch(
         'https://api.github.com/repos/Hushlor/mcp-server-tauri/releases/latest'
      );

      if (response.ok) {
         const data = await response.json();

         version.value = data.tag_name || '';
      }
   } catch{
      // Silently fail - version badge just won't show
   } finally {
      loading.value = false;
   }
});
</script>

<style scoped>
.version-badge {
   display: inline-flex;
   align-items: center;
   padding: 2px 8px;
   font-size: 12px;
   font-weight: 500;
   color: var(--vp-c-brand-1);
   background-color: var(--vp-c-brand-soft);
   border-radius: 10px;
   margin-left: 8px;
}
</style>
