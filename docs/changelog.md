---
title: Changelog
description: Release history and notable changes for MCP Server Tauri - track new features, bug fixes, and improvements across versions.
head:
  - - meta
    - name: keywords
      content: changelog, release notes, version history, updates
---

# Changelog

All notable changes to this project are documented here. Releases are fetched dynamically from [GitHub Releases](https://github.com/Hushlor/mcp-server-tauri/releases).

<script setup>
import { ref, onMounted } from 'vue';
import MarkdownIt from 'markdown-it';

const md = new MarkdownIt({ html: true, linkify: true });

const releases = ref([]);
const changelogByVersion = ref({});
const loading = ref(true);
const error = ref('');

onMounted(async () => {
   // Fetch both in parallel
   const [releasesRes, changelogRes] = await Promise.all([
      fetch('https://api.github.com/repos/Hushlor/mcp-server-tauri/releases').catch(() => null),
      fetch('https://raw.githubusercontent.com/Hushlor/mcp-server-tauri/main/CHANGELOG.md').catch(() => null),
   ]);

   // Parse releases
   if (releasesRes?.ok) {
      releases.value = await releasesRes.json();
   } else {
      error.value = 'Failed to load releases';
   }

   // Parse changelog and index by version
   if (changelogRes?.ok) {
      const content = await changelogRes.text();
      const versionRegex = /## \[(\d+\.\d+\.\d+)\][^\n]*\n([\s\S]*?)(?=## \[|$)/g;
      let match;
      while ((match = versionRegex.exec(content)) !== null) {
         const version = match[1];
         const body = match[2].trim();
         if (body && !body.startsWith('_No changes')) {
            changelogByVersion.value[version] = body;
         }
      }
   }

   loading.value = false;
});

function extractVersion(tagName) {
   // Handle tags like "v0.2.0", "tauri-mcp-server/v0.2.0", etc.
   const match = tagName.match(/v?(\d+\.\d+\.\d+)/);
   return match ? match[1] : null;
}

function getChangelogForRelease(release) {
   const version = extractVersion(release.tag_name);
   return version ? changelogByVersion.value[version] : null;
}

function formatDate(dateString) {
   return new Date(dateString).toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'long',
      day: 'numeric'
   });
}

function renderMarkdown(text) {
   if (!text) return '';
   return md.render(text);
}
</script>

<div class="releases-container">
   <div v-if="loading" class="loading">Loading releases...</div>
   <div v-else-if="error" class="error">
      {{ error }}. <a href="https://github.com/Hushlor/mcp-server-tauri/releases" target="_blank" rel="noopener">View on GitHub →</a>
   </div>
   <div v-else-if="releases.length === 0" class="empty">
      No releases found. <a href="https://github.com/Hushlor/mcp-server-tauri/releases" target="_blank" rel="noopener">View on GitHub →</a>
   </div>
   <div v-else class="releases-list">
      <div v-for="release in releases" :key="release.id" class="release-card">
         <div class="release-header">
            <a :href="release.html_url" target="_blank" rel="noopener" class="release-title">
               {{ release.name || release.tag_name }}
            </a>
            <span v-if="release.prerelease" class="prerelease-badge">Pre-release</span>
         </div>
         <div class="release-meta">
            <span class="release-tag">{{ release.tag_name }}</span>
            <span class="release-date">{{ formatDate(release.published_at) }}</span>
         </div>
         <div v-if="getChangelogForRelease(release)" class="project-changes">
            <div class="project-changes-header">Project Changes</div>
            <div v-html="renderMarkdown(getChangelogForRelease(release))"></div>
         </div>
         <div v-if="release.body" class="release-body">
            <div class="release-body-header">Package Changes</div>
            <div v-html="renderMarkdown(release.body)"></div>
         </div>
         <a :href="release.html_url" target="_blank" rel="noopener" class="view-release">
            View on GitHub →
         </a>
      </div>
   </div>
</div>

<style>
.releases-container {
   margin-top: 24px;
}

.loading, .error, .empty {
   padding: 24px;
   text-align: center;
   color: var(--vp-c-text-2);
}

.error {
   color: var(--vp-c-danger-1);
}

.releases-list {
   display: flex;
   flex-direction: column;
   gap: 24px;
}

.release-card {
   padding: 20px;
   border: 1px solid var(--vp-c-divider);
   border-radius: 12px;
   background: var(--vp-c-bg-soft);
}

.release-header {
   display: flex;
   align-items: center;
   gap: 12px;
   margin-bottom: 8px;
}

.release-title {
   font-size: 1.25rem;
   font-weight: 600;
   color: var(--vp-c-brand-1);
   text-decoration: none;
}

.release-title:hover {
   text-decoration: underline;
}

.prerelease-badge {
   padding: 2px 8px;
   font-size: 12px;
   font-weight: 500;
   color: var(--vp-c-warning-1);
   background: var(--vp-c-warning-soft);
   border-radius: 10px;
}

.release-meta {
   display: flex;
   align-items: center;
   gap: 12px;
   margin-bottom: 16px;
   font-size: 14px;
   color: var(--vp-c-text-2);
}

.release-tag {
   padding: 2px 8px;
   background: var(--vp-c-default-soft);
   border-radius: 6px;
   font-family: var(--vp-font-family-mono);
   font-size: 13px;
}

.project-changes,
.release-body {
   padding: 16px;
   background: var(--vp-c-bg);
   border-radius: 8px;
   margin-bottom: 12px;
   font-size: 14px;
   line-height: 1.6;
}

.project-changes-header,
.release-body-header {
   font-size: 0.75rem;
   font-weight: 600;
   text-transform: uppercase;
   letter-spacing: 0.05em;
   color: var(--vp-c-text-3);
   margin-bottom: 12px;
}

.project-changes h3,
.release-body h3 {
   font-size: 1rem;
   font-weight: 600;
   margin: 16px 0 8px 0;
}

.project-changes h3:first-of-type,
.release-body h3:first-of-type {
   margin-top: 0;
}

.project-changes h4,
.release-body h4 {
   font-size: 0.9rem;
   font-weight: 600;
   margin: 12px 0 6px 0;
}

.project-changes ul,
.release-body ul {
   margin: 8px 0;
   padding-left: 20px;
}

.project-changes li,
.release-body li {
   margin: 4px 0;
}

.view-release {
   font-size: 14px;
   color: var(--vp-c-brand-1);
   text-decoration: none;
}

.view-release:hover {
   text-decoration: underline;
}
</style>
