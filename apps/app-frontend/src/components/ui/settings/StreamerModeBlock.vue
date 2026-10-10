<script setup lang="ts">
import { computed } from 'vue'
import { streamerMode } from '@/store/launcherPreferences'
import i18n from '@/i18n.config'

const isRu = computed(() => (i18n.global.locale.value || '').startsWith('ru'))
</script>

<template>
	<div class="mt-8 pt-6 border-t border-[var(--er-border)]">
		<div class="flex items-start gap-3">
			<div class="text-[var(--er-text)] mt-1">
				<svg class="w-5 h-5 text-gray-200" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
					<path d="M16 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"></path>
					<circle cx="8.5" cy="7" r="4"></circle>
					<polygon points="23 7 16 12 23 17 23 7"></polygon>
				</svg>
			</div>
			<div>
				<h2 class="m-0 text-lg font-bold text-contrast">
					{{ isRu ? 'Режим стримера' : 'Streamer mode' }}
				</h2>
				<p class="m-0 mt-1 text-sm text-secondary leading-relaxed">
					{{ isRu ? 'Скрытие конфиденциальной информации от посторонних глаз.' : 'Hide sensitive information from others while streaming or sharing screen.' }}
					<br />
					{{ isRu ? 'К ним относятся: электронные почты, персональные данные, идентификаторы и прочее.' : 'This includes emails, personal profile data, IDs, and more.' }}
				</p>
			</div>
		</div>

		<div class="mt-5 grid grid-cols-1 md:grid-cols-2 gap-4">
			<div
				class="flex flex-col gap-3 cursor-pointer select-none"
				@click="streamerMode = false"
			>
				<div
					class="w-full h-36 rounded-2xl bg-[#141519] border transition-all flex flex-col items-center justify-center gap-3 p-4"
					:class="!streamerMode ? 'border-[#3b82f6] shadow-md ring-1 ring-[#3b82f6]' : 'border-[#23252d] hover:border-[#333642]'"
				>
					<div class="px-5 py-2 rounded-full bg-[#1e2026] border border-[#2d303a] flex items-center gap-2.5 text-sm text-gray-200">
						<svg class="w-4 h-4 text-gray-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
							<path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"></path>
							<circle cx="12" cy="7" r="4"></circle>
						</svg>
						<span class="font-medium text-xs">name@domain.com</span>
					</div>
					<div class="px-3.5 py-1 rounded-full bg-[#1a1c22] text-[11px] font-mono text-gray-400">
						ID: 011
					</div>
				</div>

				<div class="flex items-center gap-2.5 px-1">
					<div
						class="w-4 h-4 rounded-full border flex items-center justify-center transition-colors"
						:class="!streamerMode ? 'border-[#3b82f6] bg-[#3b82f6]' : 'border-gray-600 bg-transparent'"
					>
						<div v-if="!streamerMode" class="w-1.5 h-1.5 rounded-full bg-white"></div>
					</div>
					<span class="text-xs font-semibold" :class="!streamerMode ? 'text-white' : 'text-gray-400'">
						{{ isRu ? 'Отображать все' : 'Show everything' }}
					</span>
				</div>
			</div>

			<div
				class="flex flex-col gap-3 cursor-pointer select-none"
				@click="streamerMode = true"
			>
				<div
					class="w-full h-36 rounded-2xl bg-[#141519] border transition-all flex flex-col items-center justify-center gap-3 p-4"
					:class="streamerMode ? 'border-[#3b82f6] shadow-md ring-1 ring-[#3b82f6]' : 'border-[#23252d] hover:border-[#333642]'"
				>
					<div class="px-5 py-2 rounded-full bg-[#1e2026] border border-[#2d303a] flex items-center gap-2.5 text-sm text-gray-200">
						<svg class="w-4 h-4 text-gray-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
							<path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"></path>
							<circle cx="12" cy="7" r="4"></circle>
						</svg>
						<div class="w-32 h-2.5 rounded-full bg-[#4a4d5a]"></div>
					</div>
					<div class="px-3.5 py-1 rounded-full bg-[#1a1c22] flex items-center gap-2 text-[11px] font-mono text-gray-400">
						<span>ID:</span>
						<div class="w-8 h-2 rounded-full bg-[#4a4d5a]"></div>
					</div>
				</div>

				<div class="flex items-center gap-2.5 px-1">
					<div
						class="w-4 h-4 rounded-full border flex items-center justify-center transition-colors"
						:class="streamerMode ? 'border-[#3b82f6] bg-[#3b82f6]' : 'border-gray-600 bg-transparent'"
					>
						<div v-if="streamerMode" class="w-1.5 h-1.5 rounded-full bg-white"></div>
					</div>
					<span class="text-xs font-semibold" :class="streamerMode ? 'text-white' : 'text-gray-400'">
						{{ isRu ? 'Скрывать все' : 'Hide everything' }}
					</span>
				</div>
			</div>
		</div>
	</div>
</template>
