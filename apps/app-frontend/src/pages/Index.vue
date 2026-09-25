<script setup lang="ts">
import { injectNotificationManager } from '@erteam/ui'
import dayjs from 'dayjs'
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'

import { instance_listener } from '@/helpers/events'
import { list, run } from '@/helpers/instance'
import type { GameInstance } from '@/helpers/types'
import { useBreadcrumbs } from '@/store/breadcrumbs'
import bg1 from '@/assets/wallpapers/minecraft_bg_1.jpg'
import bg2 from '@/assets/wallpapers/minecraft_bg_2.jpg'
import bg3 from '@/assets/wallpapers/minecraft_bg_3.jpg'
import defaultMinecraftBlock from '@/assets/minecraft_block.png'

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

const wallpapers = [bg1, bg2, bg3]

const recentInstances = computed(() =>
	instances.value
		.filter((x) => x.last_played)
		.slice()
		.sort((a, b) => dayjs(b.last_played).diff(dayjs(a.last_played))),
)

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

function handleOpenInstanceSettings(instanceId: string) {
	router.push(`/instance/${instanceId}`)
}

function handleCreateNewInstance() {
	router.push('/library?action=create')
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

onMounted(async () => {
	await fetchInstances()
})

const unlistenInstance = await instance_listener(async () => {
	await fetchInstances()
})

onUnmounted(() => {
	if (typeof unlistenInstance === 'function') {
		unlistenInstance()
	}
})
</script>

<template>
	<div class="p-6 flex flex-col gap-6 max-w-7xl mx-auto">
		<!-- HERO BANNER -->
		<div class="relative w-full h-[270px] rounded-3xl overflow-hidden shadow-2xl border border-white/10 group select-none">
			<!-- Background image -->
			<img
				:src="wallpapers[currentWallpaperIndex]"
				alt="Hero Banner"
				class="absolute inset-0 w-full h-full object-cover object-center transition-transform duration-700 group-hover:scale-105"
			/>

			<!-- Gradient overlays -->
			<div class="absolute inset-0 bg-gradient-to-t from-[#141518] via-[#141518]/60 to-transparent"></div>
			<div class="absolute inset-0 bg-gradient-to-r from-[#141518]/95 via-[#141518]/50 to-transparent"></div>

			<!-- Top right quick controls -->
			<div class="absolute top-4 right-4 flex items-center gap-2 z-10">
				<!-- Mute audio button -->
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

				<!-- Change background button -->
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

			<!-- Banner Content (Bottom) -->
			<div class="absolute bottom-5 left-6 right-6 flex items-end justify-between z-10">
				<!-- Instance Details -->
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

				<!-- Actions: Settings & Play Button -->
				<div class="flex items-center gap-3">
					<!-- Settings Button -->
					<button
						v-if="activeHeroInstance"
						class="w-11 h-11 rounded-2xl bg-[#1e2025]/80 hover:bg-[#282b32] backdrop-blur border border-white/10 text-white/90 hover:text-white flex items-center justify-center transition-all cursor-pointer shadow-md"
						:title="isRu ? 'Настройки сборки' : 'Instance settings'"
						@click="handleOpenInstanceSettings(activeHeroInstance.id)"
					>
						<svg class="w-5 h-5 text-gray-300" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
							<circle cx="12" cy="12" r="3"></circle>
							<path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"></path>
						</svg>
					</button>

					<!-- Big Play Button -->
					<button
						class="flex items-center gap-2.5 px-7 py-3 rounded-2xl bg-[#22c55e] hover:bg-[#16a34a] text-[#121315] font-black text-base transition-all duration-200 cursor-pointer shadow-lg active:scale-95 border-0"
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

		<!-- SECTION: МОИ СБОРКИ -->
		<div class="flex flex-col gap-3.5">
			<div class="flex items-center justify-between">
				<h2 class="text-base font-bold text-[var(--er-text)] m-0">{{ isRu ? 'Мои сборки' : 'My instances' }}</h2>
				<button
					class="text-xs font-semibold text-[var(--er-text-secondary)] hover:text-[#22c55e] transition-colors bg-transparent border-0 cursor-pointer flex items-center gap-1"
					@click="router.push('/library')"
				>
					<span>{{ isRu ? 'Все сборки' : 'All instances' }}</span>
					<svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
						<polyline points="9 18 15 12 9 6"></polyline>
					</svg>
				</button>
			</div>

			<!-- Instances horizontal grid -->
			<div class="flex items-center gap-4 overflow-x-auto pb-2 custom-scrollbar">
				<!-- Instance Cards -->
				<div
					v-for="instance in instances"
					:key="instance.id"
					class="w-44 h-52 bg-[var(--er-card-bg)] hover:bg-[var(--er-card-hover)] border border-[var(--er-card-border)] hover:border-[#22c55e]/40 rounded-2xl p-3 flex flex-col justify-between transition-all duration-200 cursor-pointer shrink-0 group select-none shadow-sm"
					@click="handlePlay(instance)"
				>
					<!-- Top Block: 3D cube thumbnail container -->
					<div class="w-full h-28 rounded-xl bg-[var(--er-subtle-bg)] border border-white/5 flex items-center justify-center overflow-hidden relative">
						<img
							:src="instance.icon_path || defaultMinecraftBlock"
							alt="Instance icon"
							class="w-14 h-14 object-contain"
							@error="(e) => ((e.target as HTMLImageElement).src = defaultMinecraftBlock)"
						/>
					</div>

					<!-- Bottom info -->
					<div class="flex flex-col gap-0.5">
						<span class="text-xs font-bold text-[var(--er-text)] group-hover:text-[#22c55e] truncate transition-colors leading-tight">
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

				<!-- "+ Новая сборка" Card -->
				<div
					class="w-44 h-52 bg-[var(--er-subtle-bg)] hover:bg-[var(--er-card-hover)] border-2 border-dashed border-[var(--er-border)] hover:border-[#22c55e]/50 rounded-2xl flex flex-col items-center justify-center gap-2 cursor-pointer transition-all duration-200 shrink-0 text-[var(--er-text-secondary)] hover:text-[#22c55e] group select-none"
					@click="handleCreateNewInstance"
				>
					<div class="w-10 h-10 rounded-full bg-[var(--er-card-bg)] group-hover:bg-[#22c55e]/15 flex items-center justify-center transition-colors">
						<svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
							<line x1="12" y1="5" x2="12" y2="19"></line>
							<line x1="5" y1="12" x2="19" y2="12"></line>
						</svg>
					</div>
					<span class="text-xs font-semibold">{{ isRu ? 'Новая сборка' : 'New instance' }}</span>
				</div>
			</div>
		</div>
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
	background: #22c55e;
}
</style>
