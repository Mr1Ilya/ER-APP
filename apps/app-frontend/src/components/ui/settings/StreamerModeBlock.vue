<script setup lang="ts">
import { computed } from 'vue'
import { streamerMode } from '@/store/launcherPreferences'
import i18n from '@/i18n.config'

const isRu = computed(() => (i18n.global.locale.value || '').startsWith('ru'))
</script>

<template>
	<div class="mt-8 pt-6 border-t border-[var(--er-border)]">
		<div class="flex items-start gap-3">
			<div class="text-[var(--er-text)] mt-0.5">
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
				class="rounded-2xl p-5 bg-[#16181d] border transition-all cursor-pointer select-none flex flex-col justify-between h-40"
				:class="!streamerMode ? 'border-[#3b82f6] shadow-sm' : 'border-[#26282f] hover:border-[#383b45]'"
				@click="streamerMode = false"
			>
				<div class="flex flex-col items-center justify-center flex-1 gap-3">
					<div class="px-4 py-2 rounded-xl bg-[#20222a] border border-[#2b2e38] flex items-center gap-2.5 text-sm text-gray-200">
						<svg class="w-4 h-4 text-gray-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
							<path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"></path>
							<circle cx="12" cy="7" r="4"></circle>
						</svg>
						<span class="font-medium">name@domain.com</span>
					</div>
					<div class="px-3 py-1 rounded-lg bg-[#1a1c22] text-xs font-mono text-gray-400">
						ID: 011
					</div>
				</div>

				<div class="flex items-center gap-2.5 pt-2">
					<div
						class="w-4 h-4 rounded-full border flex items-center justify-center transition-colors"
						:class="!streamerMode ? 'border-[#3b82f6] bg-[#3b82f6]' : 'border-gray-600 bg-transparent'"
					>
						<div v-if="!streamerMode" class="w-1.5 h-1.5 rounded-full bg-white"></div>
					</div>
					<span class="text-xs font-medium" :class="!streamerMode ? 'text-white' : 'text-gray-400'">
						{{ isRu ? 'Отображать все' : 'Show everything' }}
					</span>
				</div>
			</div>

			<div
				class="rounded-2xl p-5 bg-[#16181d] border transition-all cursor-pointer select-none flex flex-col justify-between h-40"
				:class="streamerMode ? 'border-[#3b82f6] shadow-sm' : 'border-[#26282f] hover:border-[#383b45]'"
				@click="streamerMode = true"
			>
				<div class="flex flex-col items-center justify-center flex-1 gap-3">
					<div class="px-4 py-2 rounded-xl bg-[#20222a] border border-[#2b2e38] flex items-center gap-2.5 text-sm text-gray-200">
						<svg class="w-4 h-4 text-gray-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
							<path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"></path>
							<circle cx="12" cy="7" r="4"></circle>
						</svg>
						<div class="w-36 h-3 rounded-full bg-gray-600/70 animate-pulse"></div>
					</div>
					<div class="px-3 py-1 rounded-lg bg-[#1a1c22] flex items-center gap-1.5 text-xs font-mono text-gray-400">
						<span>ID:</span>
						<div class="w-10 h-2.5 rounded-full bg-gray-600/70"></div>
					</div>
				</div>

				<div class="flex items-center gap-2.5 pt-2">
					<div
						class="w-4 h-4 rounded-full border flex items-center justify-center transition-colors"
						:class="streamerMode ? 'border-[#3b82f6] bg-[#3b82f6]' : 'border-gray-600 bg-transparent'"
					>
						<div v-if="streamerMode" class="w-1.5 h-1.5 rounded-full bg-white"></div>
					</div>
					<span class="text-xs font-medium" :class="streamerMode ? 'text-white' : 'text-gray-400'">
						{{ isRu ? 'Скрывать все' : 'Hide everything' }}
					</span>
				</div>
			</div>
		</div>
	</div>
</template>
