<script setup lang="ts">
import { injectNotificationManager } from '@erteam/ui'
import dayjs from 'dayjs'
import { computed, inject, nextTick, onMounted, onUnmounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { convertFileSrc } from '@tauri-apps/api/core'

import { instance_listener } from '@/helpers/events'
import { list, run } from '@/helpers/instance'
import type { GameInstance } from '@/helpers/types'
import { useBreadcrumbs } from '@/store/breadcrumbs'
import {
	homeGroups,
	isInstancePinned,
	togglePinInstance,
	type HomeGroupConfig,
} from '@/store/launcherPreferences'
import defaultMinecraftBlock from '@/assets/minecraft_block.png'
import InstanceSettingsModal from '@/components/ui/modal/InstanceSettingsModal.vue'

import i18n from '@/i18n.config'

const { handleError } = injectNotificationManager()
const route = useRoute()
const router = useRouter()
const breadcrumbs = useBreadcrumbs()

const isRu = computed(() => (i18n.global.locale.value || '').startsWith('ru'))

watch(
	isRu,
	(ru) => {
		breadcrumbs.setRootContext({ name: ru ? 'Главная' : 'Home', link: route.path })
	},
	{ immediate: true },
)

const instances = ref<GameInstance[]>([])
const isLaunching = ref(false)
const launchingInstanceId = ref<string | null>(null)
const isMuted = ref(false)
const currentWallpaperIndex = ref(0)

const wallpaperModules = import.meta.glob<{ default: string }>(
	'@/assets/wallpapers/*.{png,jpg,jpeg,webp}',
	{ eager: true },
)
const wallpapers = Object.values(wallpaperModules).map((m) => m.default)

const recentInstances = computed(() =>
	instances.value
		.filter((x) => x.last_played)
		.slice()
		.sort((a, b) => dayjs(b.last_played).diff(dayjs(a.last_played))),
)

const visibleGroups = computed(() => homeGroups.value.filter((g) => g.visible))

function getInstancesForGroup(group: HomeGroupConfig) {
	if (group.id === 'recent') {
		return recentInstances.value.length ? recentInstances.value : instances.value
	}
	if (group.id === 'pinned') {
		return instances.value.filter((x) => isInstancePinned(x.id))
	}
	if (group.filterType === 'manual' && group.selectedInstanceIds) {
		return instances.value.filter((x) => group.selectedInstanceIds?.includes(x.id))
	}
	if (group.filterType === 'loader' && group.targetLoader) {
		return instances.value.filter(
			(x) => (x.loader || '').toLowerCase() === (group.targetLoader || '').toLowerCase(),
		)
	}
	if (group.id === 'releases') {
		return instances.value
			.slice()
			.sort((a, b) => b.game_version.localeCompare(a.game_version, undefined, { numeric: true }))
	}
	return instances.value
}

function getGridClass(rows: number) {
	if (rows === 1) return 'flex items-center gap-4 overflow-x-auto pb-2 custom-scrollbar'
	if (rows === 2) return 'grid grid-flow-col grid-rows-2 auto-cols-max gap-4 overflow-x-auto pb-2 custom-scrollbar'
	if (rows === 3) return 'grid grid-flow-col grid-rows-3 auto-cols-max gap-4 overflow-x-auto pb-2 custom-scrollbar'
	return 'grid grid-flow-col grid-rows-4 auto-cols-max gap-4 overflow-x-auto pb-2 custom-scrollbar'
}

const activeHeroInstance = computed<GameInstance | null>(() => {
	if (recentInstances.value.length > 0) {
		return recentInstances.value[0]
	}
	if (instances.value.length > 0) {
		return instances.value[0]
	}
	return null
})

async function fetchInstances() {
	try {
		instances.value = (await list().catch(handleError)) || []
	} catch (e) {
		console.error('Failed to fetch instances', e)
	}
}

async function handlePlay(instance: GameInstance | null) {
	if (!instance) {
		router.push('/library')
		return
	}
	isLaunching.value = true
	launchingInstanceId.value = instance.id
	try {
		await run(instance.id).catch(handleError)
	} catch (e) {
		console.error('Launch failed', e)
	} finally {
		setTimeout(() => {
			isLaunching.value = false
			launchingInstanceId.value = null
		}, 3000)
	}
}

const showCreationModal = inject<() => void>('showCreationModal')
const selectedSettingsInstance = ref<GameInstance | null>(null)
const instanceSettingsModalRef = ref<any>(null)

function handleOpenInstanceSettings(inst: GameInstance | null) {
	if (!inst) return
	selectedSettingsInstance.value = inst
	nextTick(() => {
		instanceSettingsModalRef.value?.show()
	})
}

function handleCreateNewInstance() {
	if (showCreationModal) {
		showCreationModal()
	} else {
		router.push('/library?action=create')
	}
}

function toggleWallpaper() {
	currentWallpaperIndex.value = (currentWallpaperIndex.value + 1) % wallpapers.length
}

function formatPlaytime(seconds?: number) {
	if (!seconds || seconds <= 0) return isRu.value ? 'Не запускалось' : 'Never played'
	const mins = Math.floor(seconds / 60)
	if (mins < 60) return `${mins} ${isRu.value ? 'мин' : 'min'}`
	const hours = (mins / 60).toFixed(1)
	return `${hours} ${isRu.value ? 'ч' : 'h'}`
}

let unlistenInstance: (() => void) | null = null

onMounted(async () => {
	await fetchInstances()
	try {
		unlistenInstance = await instance_listener(async () => {
			await fetchInstances()
		})
	} catch (e) {
		console.error('Failed to attach instance listener in Index.vue', e)
	}
})

onUnmounted(() => {
	if (typeof unlistenInstance === 'function') {
		unlistenInstance()
	}
})
</script>

<template>
	<div class="p-6 flex flex-col gap-6 max-w-7xl mx-auto">
		<div
			v-if="visibleGroups.some((g) => g.id === 'news')"
			class="relative w-full h-[270px] rounded-3xl overflow-hidden shadow-2xl border border-white/10 group select-none"
		>
			<img
				:src="wallpapers[currentWallpaperIndex]"
				alt="Hero Banner"
				class="absolute inset-0 w-full h-full object-cover object-center transition-transform duration-700 group-hover:scale-105"
			/>

			<div class="absolute inset-0 bg-gradient-to-t from-[#141518] via-[#141518]/60 to-transparent"></div>
			<div class="absolute inset-0 bg-gradient-to-r from-[#141518]/95 via-[#141518]/50 to-transparent"></div>

			<div class="absolute top-4 right-4 flex items-center gap-2 z-10">
				<button
					class="p-2 rounded-xl bg-black/40 hover:bg-black/60 backdrop-blur border border-white/10 text-white/80 hover:text-white transition-all cursor-pointer"
					:title="isMuted ? (isRu ? 'Включить звук' : 'Unmute') : (isRu ? 'Выключить звук' : 'Mute')"
					@click="isMuted = !isMuted"
				>
					<svg v-if="!isMuted" class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
						<polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5"></polygon>
						<path d="M19.07 4.93a10 10 0 0 1 0 14.14M15.54 8.46a5 5 0 0 1 0 7.07"></path>
					</svg>
					<svg v-else class="w-4 h-4 text-red-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
						<polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5"></polygon>
						<line x1="23" y1="9" x2="17" y2="15"></line>
						<line x1="17" y1="9" x2="23" y2="15"></line>
					</svg>
				</button>

				<button
					class="flex items-center gap-1.5 px-3 py-1.5 rounded-xl bg-black/40 hover:bg-black/60 backdrop-blur border border-white/10 text-xs font-medium text-white/90 hover:text-white transition-all cursor-pointer"
					@click="toggleWallpaper"
				>
					<svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
						<rect x="3" y="3" width="18" height="18" rx="2" ry="2"></rect>
						<circle cx="8.5" cy="8.5" r="1.5"></circle>
						<polyline points="21 15 16 10 5 21"></polyline>
					</svg>
					<span>{{ isRu ? 'Фон' : 'Background' }}</span>
				</button>
			</div>

			<div class="absolute bottom-5 left-6 right-6 flex items-end justify-between z-10">
				<div class="flex flex-col gap-1.5 max-w-xl">
					<span class="text-[11px] font-bold tracking-widest text-[#9ca3af] uppercase">
						{{ activeHeroInstance ? (isRu ? 'ПРОДОЛЖИТЬ' : 'CONTINUE') : (isRu ? 'ГОТОВ К ИГРЕ' : 'READY TO PLAY') }}
					</span>
					<h1 class="text-2xl md:text-3xl font-black text-white m-0 tracking-wide drop-shadow-md">
						{{ activeHeroInstance ? activeHeroInstance.name : 'EndRage Default' }}
					</h1>
					<div class="flex items-center gap-2 mt-0.5">
						<span class="bg-[#1e2025]/80 backdrop-blur border border-white/10 px-2.5 py-0.5 rounded-full text-xs font-semibold text-gray-200 capitalize">
							{{ activeHeroInstance ? activeHeroInstance.loader : 'Fabric' }}
						</span>
						<span class="bg-[#1e2025]/80 backdrop-blur border border-white/10 px-2.5 py-0.5 rounded-full text-xs font-semibold text-gray-200">
							{{ activeHeroInstance ? activeHeroInstance.game_version : '1.21.1' }}
						</span>
					</div>
				</div>

				<div class="flex items-center gap-3">
					<button
						v-if="activeHeroInstance"
						class="w-11 h-11 rounded-2xl bg-[#1e2025]/80 hover:bg-[#282b32] backdrop-blur border border-white/10 flex items-center justify-center transition-all cursor-pointer shadow-md"
						:class="isInstancePinned(activeHeroInstance.id) ? 'text-amber-400 border-amber-400/50 bg-amber-500/20' : 'text-gray-300 hover:text-white'"
						:title="isInstancePinned(activeHeroInstance.id) ? (isRu ? 'Открепить сборку' : 'Unpin instance') : (isRu ? 'Закрепить сборку' : 'Pin instance')"
						@click="togglePinInstance(activeHeroInstance.id)"
					>
						<svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
							<line x1="12" y1="17" x2="12" y2="22"></line>
							<path d="M5 17h14v-1.76a2 2 0 0 0-1.11-1.79l-1.78-.9A2 2 0 0 1 15 10.76V6h1a2 2 0 0 0 0-4H8a2 2 0 0 0 0 4h1v4.76a2 2 0 0 1-1.11 1.79l-1.78.9A2 2 0 0 0 5 15.24Z"></path>
						</svg>
					</button>

					<button
						v-if="activeHeroInstance"
						class="w-11 h-11 rounded-2xl bg-[#1e2025]/80 hover:bg-[#282b32] backdrop-blur border border-white/10 text-white/90 hover:text-white flex items-center justify-center transition-all cursor-pointer shadow-md"
						:title="isRu ? 'Настройки сборки' : 'Instance settings'"
						@click="handleOpenInstanceSettings(activeHeroInstance)"
					>
						<svg class="w-5 h-5 text-gray-300" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
							<circle cx="12" cy="12" r="3"></circle>
							<path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"></path>
						</svg>
					</button>

					<button
						class="flex items-center gap-2.5 px-7 py-3 rounded-2xl bg-white hover:bg-gray-100 text-[#121315] font-black text-base transition-all duration-200 cursor-pointer shadow-lg active:scale-95 border-0"
						:disabled="isLaunching"
						@click="handlePlay(activeHeroInstance)"
					>
						<svg v-if="!isLaunching" class="w-5 h-5 fill-current" viewBox="0 0 24 24">
							<polygon points="6,4 20,12 6,20"></polygon>
						</svg>
						<div v-else class="w-5 h-5 border-2 border-[#121315] border-t-transparent rounded-full animate-spin"></div>
						<span>{{ isLaunching ? (isRu ? 'Запуск...' : 'Launching...') : (isRu ? 'Играть' : 'Play') }}</span>
					</button>
				</div>
			</div>
		</div>

		<template v-for="group in visibleGroups.filter((g) => g.id !== 'news')" :key="group.id">
			<div class="flex flex-col gap-3.5">
				<div class="flex items-center justify-between">
					<div class="flex items-center gap-2">
						<h2 class="text-base font-bold text-[var(--er-text)] m-0">{{ group.title }}</h2>
						<span
							v-if="getInstancesForGroup(group).length"
							class="px-2 py-0.5 rounded-full bg-[#20222a] text-[11px] font-semibold text-gray-400"
						>
							{{ getInstancesForGroup(group).length }}
						</span>
					</div>
					<button
						class="text-xs font-semibold text-[var(--er-text-secondary)] hover:text-[var(--er-text)] transition-colors bg-transparent border-0 cursor-pointer flex items-center gap-1"
						@click="router.push('/library')"
					>
						<span>{{ isRu ? 'Все сборки' : 'All instances' }}</span>
						<svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
							<polyline points="9 18 15 12 9 6"></polyline>
						</svg>
					</button>
				</div>

				<div v-if="group.id === 'pinned' && getInstancesForGroup(group).length === 0" class="flex items-center gap-3 px-5 py-4 rounded-2xl bg-[var(--er-card-bg)] border border-dashed border-[var(--er-border)] text-sm text-[var(--er-text-secondary)]">
					<svg class="w-5 h-5 text-amber-400 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
						<line x1="12" y1="17" x2="12" y2="22"></line>
						<path d="M5 17h14v-1.76a2 2 0 0 0-1.11-1.79l-1.78-.9A2 2 0 0 1 15 10.76V6h1a2 2 0 0 0 0-4H8a2 2 0 0 0 0 4h1v4.76a2 2 0 0 1-1.11 1.79l-1.78.9A2 2 0 0 0 5 15.24Z"></path>
					</svg>
					<span>{{ isRu ? 'Нажмите на булавку 📌 на любой сборке, чтобы закрепить её здесь' : 'Click the pin icon 📌 on any instance to keep it pinned here' }}</span>
				</div>

				<div v-else :class="getGridClass(group.rows)">
					<div
						v-for="instance in getInstancesForGroup(group)"
						:key="instance.id"
						class="w-44 h-52 bg-[var(--er-card-bg)] hover:bg-[var(--er-card-hover)] border border-[var(--er-card-border)] hover:border-[var(--er-border)] rounded-2xl p-3 flex flex-col justify-between transition-all duration-200 cursor-pointer shrink-0 group select-none shadow-sm relative"
						@click="handlePlay(instance)"
					>
						<div class="w-full h-28 rounded-xl bg-[var(--er-subtle-bg)] border border-[var(--er-card-border)] flex items-center justify-center overflow-hidden relative">
							<div class="absolute top-2 right-2 flex items-center gap-1.5 z-10">
								<button
									class="w-7 h-7 rounded-lg bg-black/60 hover:bg-black/90 flex items-center justify-center transition-all border border-white/10 cursor-pointer"
									:class="isInstancePinned(instance.id) ? 'text-amber-400 border-amber-400/50 bg-amber-500/20' : 'text-gray-300 hover:text-white'"
									:title="isInstancePinned(instance.id) ? (isRu ? 'Открепить сборку' : 'Unpin instance') : (isRu ? 'Закрепить сборку' : 'Pin instance')"
									@click.stop="togglePinInstance(instance.id)"
								>
									<svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
										<line x1="12" y1="17" x2="12" y2="22"></line>
										<path d="M5 17h14v-1.76a2 2 0 0 0-1.11-1.79l-1.78-.9A2 2 0 0 1 15 10.76V6h1a2 2 0 0 0 0-4H8a2 2 0 0 0 0 4h1v4.76a2 2 0 0 1-1.11 1.79l-1.78.9A2 2 0 0 0 5 15.24Z"></path>
									</svg>
								</button>

								<button
									class="w-7 h-7 rounded-lg bg-black/60 hover:bg-black/90 text-gray-300 hover:text-white flex items-center justify-center transition-all border border-white/10 cursor-pointer"
									:title="isRu ? 'Настройки сборки' : 'Instance settings'"
									@click.stop="handleOpenInstanceSettings(instance)"
								>
									<svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
										<circle cx="12" cy="12" r="3"></circle>
										<path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"></path>
									</svg>
								</button>
							</div>

							<img
								:src="instance.icon_path ? convertFileSrc(instance.icon_path) : defaultMinecraftBlock"
								alt="Instance icon"
								class="w-14 h-14 object-contain"
								@error="(e) => ((e.target as HTMLImageElement).src = defaultMinecraftBlock)"
							/>
						</div>

						<div class="flex flex-col gap-0.5">
							<span class="text-xs font-bold text-[var(--er-text)] group-hover:text-[var(--er-text)] truncate transition-colors leading-tight">
								{{ instance.name }}
							</span>
							<span class="text-[11px] text-[var(--er-text-secondary)] truncate capitalize leading-tight">
								{{ instance.loader }} · {{ instance.game_version }}
							</span>
							<div class="flex items-center gap-1 text-[10px] text-[var(--er-text-secondary)] mt-1">
								<svg class="w-3 h-3 text-[var(--er-text-secondary)]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
									<circle cx="12" cy="12" r="10"></circle>
									<polyline points="12 6 12 12 16 14"></polyline>
								</svg>
								<span>{{ formatPlaytime(instance.submitted_time_played + instance.recent_time_played) }}</span>
							</div>
						</div>
					</div>

					<div
						class="w-44 h-52 bg-[var(--er-subtle-bg)] hover:bg-[var(--er-card-hover)] border-2 border-dashed border-[var(--er-border)] hover:border-[var(--er-accent)] rounded-2xl flex flex-col items-center justify-center gap-2 cursor-pointer transition-all duration-200 shrink-0 text-[var(--er-text-secondary)] hover:text-[var(--er-text)] group select-none"
						@click="handleCreateNewInstance"
					>
						<div class="w-10 h-10 rounded-full bg-[var(--er-card-bg)] group-hover:bg-[var(--er-card-hover)] flex items-center justify-center transition-colors">
							<svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
								<line x1="12" y1="5" x2="12" y2="19"></line>
								<line x1="5" y1="12" x2="19" y2="12"></line>
							</svg>
						</div>
						<span class="text-xs font-semibold">{{ isRu ? 'Новая сборка' : 'New instance' }}</span>
					</div>
				</div>
			</div>
		</template>

		<InstanceSettingsModal
			v-if="selectedSettingsInstance"
			:key="selectedSettingsInstance.id"
			ref="instanceSettingsModalRef"
			:instance="selectedSettingsInstance"
			:offline="false"
			@unlinked="fetchInstances"
		/>
	</div>
</template>

<style scoped>
.custom-scrollbar::-webkit-scrollbar {
	height: 6px;
}
.custom-scrollbar::-webkit-scrollbar-track {
	background: transparent;
}
.custom-scrollbar::-webkit-scrollbar-thumb {
	background: #2a2c34;
	border-radius: 9999px;
}
.custom-scrollbar::-webkit-scrollbar-thumb:hover {
	background: #8b5cf6;
}
</style>
