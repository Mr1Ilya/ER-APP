<script setup lang="ts">
import { computed } from 'vue'
import { streamerMode } from '@/store/launcherPreferences'
import i18n from '@/i18n.config'

const isRu = computed(() => (i18n.global.locale.value || '').startsWith('ru'))
</script>

<template>
	<div class="mt-8 pt-6 border-t border-divider">
		<div class="flex items-start gap-3">
			<div class="text-contrast mt-1">
				<svg class="w-5 h-5 text-secondary" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
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
					class="w-full h-36 rounded-2xl bg-surface-2 transition-all flex flex-col items-center justify-center gap-3 p-4 border-2 border-solid"
					:class="!streamerMode ? 'border-purple-500 bg-[rgba(172,81,251,0.18)]' : 'border-white/10 hover:bg-surface-3'"
				>
					<div class="px-5 py-2 rounded-full bg-surface-1 border border-divider flex items-center gap-2.5 text-sm text-contrast">
						<svg class="w-4 h-4 text-secondary" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
							<path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"></path>
							<circle cx="12" cy="7" r="4"></circle>
						</svg>
						<span class="font-medium text-xs">name@domain.com</span>
					</div>
					<div class="px-3.5 py-1 rounded-full bg-surface-1 text-[11px] font-mono text-secondary">
						ID: 011
					</div>
				</div>

				<div class="flex items-center gap-2.5 px-1">
					<div
						class="w-4 h-4 rounded-full border flex items-center justify-center transition-colors"
						:class="!streamerMode ? 'border-brand bg-brand' : 'border-divider bg-transparent'"
					>
						<div v-if="!streamerMode" class="w-1.5 h-1.5 rounded-full bg-white"></div>
					</div>
					<span class="text-xs font-semibold" :class="!streamerMode ? 'text-contrast' : 'text-secondary'">
						{{ isRu ? 'Отображать все' : 'Show everything' }}
					</span>
				</div>
			</div>

			<div
				class="flex flex-col gap-3 cursor-pointer select-none"
				@click="streamerMode = true"
			>
				<div
					class="w-full h-36 rounded-2xl bg-surface-2 transition-all flex flex-col items-center justify-center gap-3 p-4 border-2 border-solid"
					:class="streamerMode ? 'border-purple-500 bg-[rgba(172,81,251,0.18)]' : 'border-white/10 hover:bg-surface-3'"
				>
					<div class="px-5 py-2 rounded-full bg-surface-1 border border-divider flex items-center gap-2.5 text-sm text-contrast">
						<svg class="w-4 h-4 text-secondary" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
							<path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"></path>
							<circle cx="12" cy="7" r="4"></circle>
						</svg>
						<div class="w-32 h-2.5 rounded-full bg-surface-4"></div>
					</div>
					<div class="px-3.5 py-1 rounded-full bg-surface-1 flex items-center gap-2 text-[11px] font-mono text-secondary">
						<span>ID:</span>
						<div class="w-8 h-2 rounded-full bg-surface-4"></div>
					</div>
				</div>

				<div class="flex items-center gap-2.5 px-1">
					<div
						class="w-4 h-4 rounded-full border flex items-center justify-center transition-colors"
						:class="streamerMode ? 'border-brand bg-brand' : 'border-divider bg-transparent'"
					>
						<div v-if="streamerMode" class="w-1.5 h-1.5 rounded-full bg-white"></div>
					</div>
					<span class="text-xs font-semibold" :class="streamerMode ? 'text-contrast' : 'text-secondary'">
						{{ isRu ? 'Скрывать все' : 'Hide everything' }}
					</span>
				</div>
			</div>
		</div>
	</div>
</template>
