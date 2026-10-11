<script setup lang="ts">
import { injectNotificationManager, Toggle } from '@erteam/ui'
import dayjs from 'dayjs'
import { computed, inject, nextTick, onMounted, onUnmounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { convertFileSrc } from '@tauri-apps/api/core'

import { instance_listener } from '@/helpers/events'
import { list, run } from '@/helpers/instance'
import type { GameInstance } from '@/helpers/types'
import { useBreadcrumbs } from '@/store/breadcrumbs'
import {
	addCustomHomeGroup,
	deleteHomeGroup,
	homeGroups,
	isInstancePinned,
	moveHomeGroup,
	toggleHomeGroupVisibility,
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
const isEditMode = ref(false)
const activeMenuGroupId = ref<string | null>(null)

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

const displayedGroups = computed(() => {
	if (isEditMode.value) {
		return homeGroups.value
	}
	return homeGroups.value.filter((g) => g.visible)
})

const hiddenGroups = computed(() => homeGroups.value.filter((g) => !g.visible))

function getInstancesForGroup(group: HomeGroupConfig) {
	let list: GameInstance[] = []
	if (group.id === 'recent') {
		list = recentInstances.value.length ? recentInstances.value : instances.value
	} else if (group.id === 'pinned') {
		list = instances.value.filter((x) => isInstancePinned(x.id))
	} else if (group.filterType === 'manual' && group.selectedInstanceIds) {
		list = instances.value.filter((x) => group.selectedInstanceIds?.includes(x.id))
	} else if (group.filterType === 'loader' && group.targetLoader) {
		list = instances.value.filter(
			(x) => (x.loader || '').toLowerCase() === (group.targetLoader || '').toLowerCase(),
		)
	} else if (group.id === 'releases') {
		list = instances.value
			.slice()
			.sort((a, b) => b.game_version.localeCompare(a.game_version, undefined, { numeric: true }))
	} else {
		list = instances.value
	}
	if (group.rows && group.rows > 1) {
		return list.slice(0, group.rows * 5)
	}
	return list
}

function getInstanceCover(instance: GameInstance): string {
	if (instance.icon_path) {
		return convertFileSrc(instance.icon_path)
	}
	let hash = 0
	for (let i = 0; i < (instance.name || '').length; i++) {
		hash = (hash << 5) - hash + instance.name.charCodeAt(i)
		hash |= 0
	}
	const idx = Math.abs(hash) % wallpapers.length
	return wallpapers[idx] || defaultMinecraftBlock
}

function getGridClass(rows: number) {
	if (rows === 1) return 'flex items-center gap-4 overflow-x-auto pb-2 custom-scrollbar flex-nowrap'
	return 'flex flex-wrap gap-4 pb-2'
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

const editingGroup = ref<HomeGroupConfig | null>(null)
const tempRows = ref<1 | 2 | 3 | 4>(2)
const tempMultiplayer = ref(true)
const tempSingleplayer = ref(true)
const tempFilterType = ref<'all' | 'manual' | 'version' | 'loader'>('all')
const tempSelectedIds = ref<string[]>([])
const tempTargetLoader = ref<string>('fabric')
const tempTargetVersion = ref<string>('')

const showCreateGroupModal = ref(false)
const newGroupTitle = ref('')
const newGroupFilterType = ref<'all' | 'manual' | 'version' | 'loader'>('all')
const newGroupSelectedIds = ref<string[]>([])
const newGroupTargetLoader = ref<string>('fabric')
const newGroupTargetVersion = ref<string>('')

function openGroupConfigModal(group: HomeGroupConfig) {
	activeMenuGroupId.value = null
	editingGroup.value = group
	tempRows.value = group.rows || 2
	tempMultiplayer.value = group.activityMultiplayer ?? true
	tempSingleplayer.value = group.activitySingleplayer ?? true
	tempFilterType.value = group.filterType || 'all'
	tempSelectedIds.value = [...(group.selectedInstanceIds || [])]
	tempTargetLoader.value = group.targetLoader || 'fabric'
	tempTargetVersion.value = group.targetVersion || ''
}

function closeGroupConfigModal() {
	editingGroup.value = null
}

function saveGroupConfig() {
	if (!editingGroup.value) return
	editingGroup.value.rows = tempRows.value
	editingGroup.value.activityMultiplayer = tempMultiplayer.value
	editingGroup.value.activitySingleplayer = tempSingleplayer.value
	editingGroup.value.filterType = tempFilterType.value
	editingGroup.value.selectedInstanceIds = [...tempSelectedIds.value]
	editingGroup.value.targetLoader = tempTargetLoader.value
	editingGroup.value.targetVersion = tempTargetVersion.value
	if (editingGroup.value.filterType === 'manual') {
		editingGroup.value.badge = tempSelectedIds.value.length
	}
	editingGroup.value = null
}

function handleCreateNewGroup() {
	if (!newGroupTitle.value.trim()) return
	addCustomHomeGroup({
		title: newGroupTitle.value.trim(),
		filterType: newGroupFilterType.value,
		selectedInstanceIds: [...newGroupSelectedIds.value],
		targetLoader: newGroupTargetLoader.value,
		targetVersion: newGroupTargetVersion.value,
	})
	newGroupTitle.value = ''
	newGroupFilterType.value = 'all'
	newGroupSelectedIds.value = []
	showCreateGroupModal.value = false
}

function toggleManualSelection(id: string, listRef: string[]) {
	const idx = listRef.indexOf(id)
	if (idx === -1) {
		listRef.push(id)
	} else {
		listRef.splice(idx, 1)
	}
}

function toggleGroupMenu(groupId: string) {
	activeMenuGroupId.value = activeMenuGroupId.value === groupId ? null : groupId
}

function handleMoveGroupUp(index: number) {
	if (index <= 1) return
	moveHomeGroup(index, index - 1)
	activeMenuGroupId.value = null
}

function handleMoveGroupDown(index: number) {
	if (index === 0 || index >= homeGroups.value.length - 1) return
	moveHomeGroup(index, index + 1)
	activeMenuGroupId.value = null
}

function handleHideGroup(groupId: string) {
	toggleHomeGroupVisibility(groupId)
	activeMenuGroupId.value = null
}

function handleDeleteGroupClick(groupId: string) {
	deleteHomeGroup(groupId)
	activeMenuGroupId.value = null
}

function onGlobalKeydown(e: KeyboardEvent) {
	if (e.key === 'Escape') {
		if (editingGroup.value) closeGroupConfigModal()
		if (showCreateGroupModal.value) showCreateGroupModal.value = false
		if (activeMenuGroupId.value) activeMenuGroupId.value = null
	}
}

function onGlobalClick(e: MouseEvent) {
	const target = e.target as HTMLElement
	if (!target.closest('.group-menu-trigger') && !target.closest('.group-menu-popover')) {
		activeMenuGroupId.value = null
	}
}

let unlistenInstance: (() => void) | null = null

onMounted(async () => {
	window.addEventListener('keydown', onGlobalKeydown)
	window.addEventListener('click', onGlobalClick)
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
	window.removeEventListener('keydown', onGlobalKeydown)
	window.removeEventListener('click', onGlobalClick)
	if (typeof unlistenInstance === 'function') {
		unlistenInstance()
	}
})
</script>

<template>
	<div class="p-6 flex flex-col gap-6 max-w-7xl mx-auto pb-24">
		<div
			v-if="displayedGroups.some((g) => g.id === 'news' && (isEditMode || g.visible))"
			class="relative w-full h-[270px] rounded-3xl overflow-hidden shadow-2xl border border-white/10 group select-none transition-all"
			:class="isEditMode ? 'ring-2 ring-blue-500/50' : ''"
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
						:title="isRu ? 'Контент сборки' : 'Instance content'"
						@click="router.push('/instance/' + encodeURIComponent(activeHeroInstance.id))"
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

		<template v-for="(group, gIndex) in displayedGroups.filter((g) => g.id !== 'news')" :key="group.id">
			<div
				v-if="isEditMode || group.visible"
				class="flex flex-col gap-3.5 transition-all relative"
				:class="[
					isEditMode ? 'border-2 border-dashed border-[#3a3d4a] rounded-3xl p-5 bg-[#14151a]/40' : '',
					!group.visible && isEditMode ? 'opacity-50' : ''
				]"
			>
				<div class="flex items-center justify-between">
					<div class="flex items-center gap-2.5">
						<div v-if="isEditMode" class="text-gray-500 hover:text-gray-300 p-0.5 cursor-grab">
							<svg class="w-4 h-4" viewBox="0 0 24 24" fill="currentColor">
								<circle cx="9" cy="5" r="1.5"></circle>
								<circle cx="15" cy="5" r="1.5"></circle>
								<circle cx="9" cy="12" r="1.5"></circle>
								<circle cx="15" cy="12" r="1.5"></circle>
								<circle cx="9" cy="19" r="1.5"></circle>
								<circle cx="15" cy="19" r="1.5"></circle>
							</svg>
						</div>

						<div class="text-gray-400">
							<svg v-if="group.icon === 'zap'" class="w-4 h-4 text-amber-400 fill-current" viewBox="0 0 24 24">
								<polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"></polygon>
							</svg>
							<svg v-else-if="group.icon === 'creeper'" class="w-4 h-4 text-emerald-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
								<rect x="3" y="3" width="18" height="18" rx="2" ry="2"></rect>
								<path d="M8 8h2v3H8zM14 8h2v3h-2zM10 13h4v4h-4zM8 17h2v2H8zM14 17h2v2h-2z"></path>
							</svg>
							<svg v-else-if="group.icon === 'pin'" class="w-4 h-4 text-sky-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
								<line x1="12" y1="17" x2="12" y2="22"></line>
								<path d="M5 17h14v-1.76a2 2 0 0 0-1.11-1.79l-1.78-.9A2 2 0 0 1 15 10.76V6h1a2 2 0 0 0 0-4H8a2 2 0 0 0 0 4h1v4.76a2 2 0 0 1-1.11 1.79l-1.78.9A2 2 0 0 0 5 15.24Z"></path>
							</svg>
							<svg v-else class="w-4 h-4 text-purple-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
								<path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path>
							</svg>
						</div>

						<h2 class="text-base font-bold text-[var(--er-text)] m-0">{{ group.title }}</h2>
						<span
							v-if="getInstancesForGroup(group).length"
							class="px-2 py-0.5 rounded-full bg-[#20222a] text-[11px] font-semibold text-gray-400"
						>
							{{ getInstancesForGroup(group).length }}
						</span>
					</div>

					<div v-if="!isEditMode">
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

					<div v-else class="flex items-center gap-1.5 relative">
						<button
							class="w-8 h-8 rounded-xl bg-[#1c1e24] hover:bg-[#252831] text-gray-400 hover:text-white flex items-center justify-center border border-white/5 cursor-pointer transition-colors"
							:title="isRu ? 'Настроить блок' : 'Configure block'"
							@click="openGroupConfigModal(group)"
						>
							<svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
								<circle cx="12" cy="12" r="3"></circle>
								<path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"></path>
							</svg>
						</button>

						<button
							class="w-8 h-8 rounded-xl bg-[#1c1e24] hover:bg-[#252831] text-gray-400 hover:text-white flex items-center justify-center border border-white/5 cursor-pointer transition-colors"
							:title="group.visible ? (isRu ? 'Скрыть блок' : 'Hide block') : (isRu ? 'Показать блок' : 'Show block')"
							@click="toggleHomeGroupVisibility(group.id)"
						>
							<svg v-if="group.visible" class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
								<path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"></path>
								<circle cx="12" cy="12" r="3"></circle>
							</svg>
							<svg v-else class="w-4 h-4 text-red-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
								<path d="M17.94 17.94A10.07 10.07 0 0 1 12 20c-7 0-11-8-11-8a18.45 18.45 0 0 1 5.06-5.94M9.9 4.24A9.12 9.12 0 0 1 12 4c7 0 11 8 11 8a18.5 18.5 0 0 1-2.16 3.19m-6.72-1.07a3 3 0 1 1-4.24-4.24"></path>
								<line x1="1" y1="1" x2="23" y2="23"></line>
							</svg>
						</button>

						<div class="relative">
							<button
								class="group-menu-trigger w-8 h-8 rounded-xl bg-surface-2 hover:bg-surface-3 text-secondary hover:text-contrast flex items-center justify-center border border-divider cursor-pointer transition-colors"
								@click.stop="toggleGroupMenu(group.id)"
							>
								<svg class="w-4 h-4 fill-current" viewBox="0 0 24 24">
									<circle cx="5" cy="12" r="2"></circle>
									<circle cx="12" cy="12" r="2"></circle>
									<circle cx="19" cy="12" r="2"></circle>
								</svg>
							</button>

							<div
								v-if="activeMenuGroupId === group.id"
								class="group-menu-popover absolute top-10 right-0 z-50 w-52 rounded-2xl bg-bg-raised border border-divider shadow-2xl p-2 flex flex-col gap-1 text-xs select-none"
								@click.stop
							>
								<button
									class="flex items-center gap-2.5 px-3 py-2 rounded-xl text-contrast/90 hover:text-contrast hover:bg-surface-3 transition-colors text-left border-0 bg-transparent cursor-pointer"
									@click="openGroupConfigModal(group)"
								>
									<svg class="w-3.5 h-3.5 text-secondary" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
										<circle cx="12" cy="12" r="3"></circle>
										<path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"></path>
									</svg>
									<span>{{ isRu ? 'Настроить блок' : 'Configure block' }}</span>
								</button>

								<div class="grid grid-cols-2 gap-1 py-1 border-y border-divider">
									<button
										class="flex items-center gap-1.5 px-2 py-1.5 rounded-lg text-contrast/90 hover:text-contrast hover:bg-surface-3 transition-colors border-0 bg-transparent cursor-pointer disabled:opacity-30 disabled:cursor-not-allowed"
										:disabled="gIndex <= 0"
										@click="handleMoveGroupUp(gIndex)"
									>
										<svg class="w-3 h-3 text-secondary" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
											<polyline points="18 15 12 9 6 15"></polyline>
										</svg>
										<span>{{ isRu ? 'Выше' : 'Up' }}</span>
									</button>
									<button
										class="flex items-center gap-1.5 px-2 py-1.5 rounded-lg text-contrast/90 hover:text-contrast hover:bg-surface-3 transition-colors border-0 bg-transparent cursor-pointer disabled:opacity-30 disabled:cursor-not-allowed"
										:disabled="gIndex >= displayedGroups.length - 1"
										@click="handleMoveGroupDown(gIndex)"
									>
										<svg class="w-3 h-3 text-secondary" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
											<polyline points="6 9 12 15 18 9"></polyline>
										</svg>
										<span>{{ isRu ? 'Ниже' : 'Down' }}</span>
									</button>
								</div>

								<button
									class="flex items-center gap-2.5 px-3 py-2 rounded-xl text-contrast/90 hover:text-contrast hover:bg-surface-3 transition-colors text-left border-0 bg-transparent cursor-pointer"
									@click="handleHideGroup(group.id)"
								>
									<svg class="w-3.5 h-3.5 text-secondary" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
										<path d="M17.94 17.94A10.07 10.07 0 0 1 12 20c-7 0-11-8-11-8a18.45 18.45 0 0 1 5.06-5.94M9.9 4.24A9.12 9.12 0 0 1 12 4c7 0 11 8 11 8a18.5 18.5 0 0 1-2.16 3.19m-6.72-1.07a3 3 0 1 1-4.24-4.24"></path>
										<line x1="1" y1="1" x2="23" y2="23"></line>
									</svg>
									<span>{{ isRu ? 'Скрыть блок' : 'Hide block' }}</span>
								</button>

								<button
									v-if="!group.locked"
									class="flex items-center gap-2.5 px-3 py-2 rounded-xl text-red-400 hover:text-red-300 hover:bg-red-500/10 transition-colors text-left border-0 bg-transparent cursor-pointer"
									@click="handleDeleteGroupClick(group.id)"
								>
									<svg class="w-3.5 h-3.5 text-red-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
										<polyline points="3 6 5 6 21 6"></polyline>
										<path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path>
									</svg>
									<span>{{ isRu ? 'Удалить группу' : 'Delete group' }}</span>
								</button>

								<div class="px-3 py-1.5 mt-1 rounded-xl bg-surface-2 text-[10px] text-secondary flex items-center gap-1.5 border border-divider">
									<svg class="w-3 h-3 text-secondary shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
										<path d="M12 20h9"></path>
										<path d="M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4L16.5 3.5z"></path>
									</svg>
									<span>{{ isRu ? 'Вернуть можно в режиме настройки главной' : 'Can restore in home customize mode' }}</span>
								</div>
							</div>
						</div>
					</div>
				</div>

				<div v-if="group.id === 'pinned' && getInstancesForGroup(group).length === 0" class="flex flex-col md:flex-row items-center justify-between gap-4 px-6 py-6 rounded-2xl bg-surface-1 border border-dashed border-divider text-sm text-secondary">
					<div class="flex items-center gap-3">
						<div class="w-10 h-10 rounded-2xl bg-amber-500/10 border border-amber-500/20 text-amber-400 flex items-center justify-center shrink-0">
							<svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
								<line x1="12" y1="17" x2="12" y2="22"></line>
								<path d="M5 17h14v-1.76a2 2 0 0 0-1.11-1.79l-1.78-.9A2 2 0 0 1 15 10.76V6h1a2 2 0 0 0 0-4H8a2 2 0 0 0 0 4h1v4.76a2 2 0 0 1-1.11 1.79l-1.78.9A2 2 0 0 0 5 15.24Z"></path>
							</svg>
						</div>
						<div>
							<h4 class="text-sm font-bold text-contrast m-0">{{ isRu ? 'Закрепленных сборок пока нет' : 'No pinned instances yet' }}</h4>
							<p class="text-xs text-secondary m-0 mt-0.5">
								{{ isRu ? 'Нажмите на булавку на любой сборке, чтобы закрепить её здесь' : 'Click the pin icon on any instance to keep it pinned here' }}
							</p>
						</div>
					</div>
					<button
						class="px-4 py-2 rounded-xl bg-surface-2 hover:bg-surface-3 border border-divider text-xs font-semibold text-contrast cursor-pointer transition-colors"
						@click="router.push('/library')"
					>
						{{ isRu ? '+ Закрепить из библиотеки' : '+ Pin from library' }}
					</button>
				</div>

				<div v-else :class="getGridClass(group.rows)">
					<div
						v-for="instance in getInstancesForGroup(group)"
						:key="instance.id"
						class="w-56 h-60 bg-[var(--er-card-bg)] hover:bg-[var(--er-card-hover)] border border-white/20 hover:border-white/40 rounded-2xl p-2.5 flex flex-col justify-between transition-all duration-200 cursor-pointer shrink-0 group select-none shadow-lg relative"
						@click="handlePlay(instance)"
					>
						<div class="w-full h-36 rounded-xl bg-surface-2 border border-white/10 overflow-hidden relative">
							<img
								:src="getInstanceCover(instance)"
								alt="Instance cover"
								class="w-full h-full object-cover object-center transition-transform duration-500 group-hover:scale-105"
								@error="(e) => ((e.target as HTMLImageElement).src = wallpapers[0] || defaultMinecraftBlock)"
							/>

							<div class="absolute inset-0 bg-gradient-to-t from-black/80 via-transparent to-black/30 pointer-events-none"></div>

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
									:title="isRu ? 'Контент сборки' : 'Instance content'"
									@click.stop="router.push('/instance/' + encodeURIComponent(instance.id))"
								>
									<svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
										<circle cx="12" cy="12" r="3"></circle>
										<path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"></path>
									</svg>
								</button>
							</div>

							<div class="absolute bottom-2 left-2 w-6 h-6 rounded-lg bg-black/60 backdrop-blur border border-white/10 flex items-center justify-center text-gray-200 z-10">
								<svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
									<rect x="3" y="3" width="18" height="18" rx="2" ry="2"></rect>
									<path d="M8 8h2v3H8zM14 8h2v3h-2zM10 13h4v4h-4zM8 17h2v2H8zM14 17h2v2h-2z"></path>
								</svg>
							</div>
						</div>

						<div class="flex items-center justify-between gap-2 pt-1.5 px-1">
							<div class="flex flex-col min-w-0 flex-1">
								<span class="text-xs font-bold text-contrast group-hover:text-brand truncate transition-colors leading-tight">
									{{ instance.name }}
								</span>
								<span class="text-[11px] text-secondary truncate capitalize leading-tight mt-0.5">
									{{ instance.loader }} · {{ instance.game_version }}
								</span>
							</div>

							<button
								class="w-8 h-8 rounded-xl bg-brand hover:brightness-110 flex items-center justify-center text-white transition-all shadow-md active:scale-95 border-0 cursor-pointer shrink-0"
								:title="isRu ? 'Играть' : 'Play'"
								@click.stop="handlePlay(instance)"
							>
								<div v-if="isLaunching && launchingInstanceId === instance.id" class="w-3.5 h-3.5 border-2 border-white border-t-transparent rounded-full animate-spin"></div>
								<svg v-else class="w-3.5 h-3.5 fill-current ml-0.5" viewBox="0 0 24 24">
									<polygon points="6,4 20,12 6,20"></polygon>
								</svg>
							</button>
						</div>
					</div>

					<div
						class="w-56 h-60 bg-surface-1 hover:bg-surface-2 border-2 border-dashed border-white/20 hover:border-purple-400 rounded-2xl flex flex-col items-center justify-center gap-2.5 cursor-pointer transition-all duration-200 shrink-0 text-secondary hover:text-contrast group select-none"
						@click="handleCreateNewInstance"
					>
						<div class="w-11 h-11 rounded-2xl bg-surface-2 group-hover:bg-surface-3 flex items-center justify-center transition-colors border border-white/10">
							<svg class="w-5 h-5 text-secondary group-hover:text-contrast transition-colors" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
								<line x1="12" y1="5" x2="12" y2="19"></line>
								<line x1="5" y1="12" x2="19" y2="12"></line>
							</svg>
						</div>
						<span class="text-xs font-bold">{{ isRu ? 'Новая сборка' : 'New instance' }}</span>
					</div>
				</div>

				<div v-if="isEditMode" class="flex items-center justify-center gap-3 pt-2">
					<button
						class="flex items-center gap-1.5 px-3.5 py-1.5 rounded-xl bg-surface-2 hover:bg-surface-3 text-xs font-medium text-secondary hover:text-contrast border border-divider cursor-pointer disabled:opacity-30 disabled:cursor-not-allowed transition-colors"
						:disabled="group.rows <= 1"
						@click="group.rows = Math.max(1, group.rows - 1)"
					>
						<svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
							<polyline points="3 6 5 6 21 6"></polyline>
							<path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path>
						</svg>
						<span>{{ isRu ? 'Убрать ряд' : 'Remove row' }}</span>
					</button>

					<span class="text-xs font-semibold text-secondary font-mono">
						{{ group.rows }} {{ isRu ? 'из 4 рядов' : 'of 4 rows' }}
					</span>

					<button
						class="flex items-center gap-1.5 px-3.5 py-1.5 rounded-xl bg-surface-2 hover:bg-surface-3 text-xs font-medium text-secondary hover:text-contrast border border-divider cursor-pointer disabled:opacity-30 disabled:cursor-not-allowed transition-colors"
						:disabled="group.rows >= 4"
						@click="group.rows = Math.min(4, group.rows + 1)"
					>
						<svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
							<line x1="12" y1="5" x2="12" y2="19"></line>
							<line x1="5" y1="12" x2="19" y2="12"></line>
						</svg>
						<span>{{ isRu ? 'Добавить ряд' : 'Add row' }}</span>
					</button>
				</div>
			</div>
		</template>

		<div
			v-if="isEditMode && hiddenGroups.length > 0"
			class="border-2 border-dashed border-divider rounded-3xl p-5 bg-surface-1/40 flex flex-col gap-3"
		>
			<h3 class="text-xs font-bold text-secondary m-0 uppercase tracking-wider">
				{{ isRu ? 'Скрытые блоки' : 'Hidden blocks' }}
			</h3>
			<div class="flex flex-wrap gap-2.5">
				<div
					v-for="hg in hiddenGroups"
					:key="hg.id"
					class="flex items-center gap-2.5 px-3.5 py-2 rounded-xl bg-surface-2 border border-divider text-xs text-contrast"
				>
					<span class="font-medium">{{ hg.title }}</span>
					<button
						class="px-2.5 py-1 rounded-lg bg-brand/15 hover:bg-brand/25 text-brand text-xs font-semibold border border-brand/20 cursor-pointer transition-colors"
						@click="toggleHomeGroupVisibility(hg.id)"
					>
						{{ isRu ? 'Показать' : 'Show' }}
					</button>
				</div>
			</div>
		</div>

		<div
			v-if="isEditMode"
			class="w-full py-4 border-2 border-dashed border-divider hover:border-brand/50 rounded-3xl flex items-center justify-center gap-2 text-xs font-semibold text-secondary hover:text-contrast cursor-pointer transition-colors"
			@click="showCreateGroupModal = true"
		>
			<svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
				<line x1="12" y1="5" x2="12" y2="19"></line>
				<line x1="5" y1="12" x2="19" y2="12"></line>
			</svg>
			<span>{{ isRu ? 'Создать группу' : 'Create group' }}</span>
		</div>

		<div v-if="!isEditMode" class="flex items-center justify-center pt-4">
			<button
				class="flex items-center gap-2 px-5 py-2.5 rounded-2xl bg-surface-2 hover:bg-surface-3 border border-divider text-xs font-semibold text-secondary hover:text-contrast transition-all cursor-pointer shadow-md"
				@click="isEditMode = true"
			>
				<svg class="w-4 h-4 text-secondary" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
					<circle cx="12" cy="12" r="3"></circle>
					<path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"></path>
				</svg>
				<span>{{ isRu ? 'Настроить главную' : 'Customize home' }}</span>
			</button>
		</div>

		<Transition name="fade">
			<div
				v-if="isEditMode"
				class="fixed bottom-6 left-1/2 -translate-x-1/2 z-40 flex items-center gap-3 px-4 py-2.5 rounded-2xl bg-bg-raised/95 backdrop-blur-xl border border-divider shadow-2xl"
			>
				<button
					class="px-4 py-2 rounded-xl bg-surface-2 hover:bg-surface-3 text-xs font-semibold text-contrast border border-divider cursor-pointer transition-colors"
					@click="showCreateGroupModal = true"
				>
					{{ isRu ? '+ Создать группу' : '+ Create group' }}
				</button>
				<button
					class="px-5 py-2 rounded-xl bg-brand hover:brightness-110 text-xs font-bold text-white border-0 cursor-pointer shadow-md flex items-center gap-1.5 transition-all active:scale-95"
					@click="isEditMode = false"
				>
					<svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3">
						<polyline points="20 6 9 17 4 12"></polyline>
					</svg>
					<span>{{ isRu ? 'Готово' : 'Done' }}</span>
				</button>
			</div>
		</Transition>

		<Teleport to="body">
			<Transition name="fade">
				<div
					v-if="editingGroup"
					class="fixed inset-0 z-[200] flex items-center justify-center bg-black/75 backdrop-blur-md p-4 overflow-y-auto select-none"
					@click.self="closeGroupConfigModal"
				>
					<div
						class="w-full max-w-2xl bg-bg-raised border border-white/10 rounded-3xl p-6 shadow-2xl flex flex-col gap-5 text-contrast my-auto"
						@click.stop
					>
						<div class="flex items-center justify-between pb-3 border-b border-divider">
							<div class="flex items-center gap-3">
								<div class="w-10 h-10 rounded-2xl bg-brand/15 text-brand flex items-center justify-center border border-brand/20">
									<svg v-if="editingGroup.icon === 'zap'" class="w-5 h-5 fill-current" viewBox="0 0 24 24">
										<polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"></polygon>
									</svg>
									<svg v-else-if="editingGroup.icon === 'creeper'" class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
										<rect x="3" y="3" width="18" height="18" rx="2" ry="2"></rect>
										<path d="M8 8h2v3H8zM14 8h2v3h-2zM10 13h4v4h-4zM8 17h2v2H8zM14 17h2v2h-2z"></path>
									</svg>
									<svg v-else-if="editingGroup.icon === 'pin'" class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
										<line x1="12" y1="17" x2="12" y2="22"></line>
										<path d="M5 17h14v-1.76a2 2 0 0 0-1.11-1.79l-1.78-.9A2 2 0 0 1 15 10.76V6h1a2 2 0 0 0 0-4H8a2 2 0 0 0 0 4h1v4.76a2 2 0 0 1-1.11 1.79l-1.78.9A2 2 0 0 0 5 15.24Z"></path>
									</svg>
									<svg v-else class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
										<path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path>
									</svg>
								</div>
								<div>
									<h3 class="text-lg font-bold text-contrast m-0">
										{{ editingGroup.title }}
									</h3>
									<p class="text-xs text-secondary m-0">
										{{ isRu ? 'Настройка отображения группы на главной' : 'Group display settings on home page' }}
									</p>
								</div>
							</div>

							<button
								class="w-8 h-8 rounded-xl bg-transparent hover:bg-surface-3 text-secondary hover:text-contrast flex items-center justify-center transition-colors border-0 cursor-pointer"
								:title="isRu ? 'Закрыть (Esc)' : 'Close (Esc)'"
								@click="closeGroupConfigModal"
							>
								<svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
									<line x1="18" y1="6" x2="6" y2="18"></line>
									<line x1="6" y1="6" x2="18" y2="18"></line>
								</svg>
							</button>
						</div>

						<div class="flex flex-col gap-4">
							<div class="flex flex-col gap-2">
								<label class="text-xs font-semibold text-secondary">
									{{ isRu ? 'Количество рядов для отображения' : 'Number of rows to display' }}
								</label>
								<div class="grid grid-cols-4 gap-2.5">
									<button
										v-for="r in [1, 2, 3, 4] as const"
										:key="r"
										type="button"
										class="py-2.5 px-3 rounded-2xl text-center flex flex-col items-center justify-center gap-1 transition-all cursor-pointer"
										:class="tempRows === r ? 'bg-brand/15 border-2 border-brand text-white font-bold' : 'bg-surface-2 border-2 border-white/10 hover:bg-surface-3 text-secondary hover:text-contrast'"
										@click="tempRows = r"
									>
										<span class="text-xs font-bold">{{ r }} {{ isRu ? (r === 1 ? 'ряд' : (r < 5 ? 'ряда' : 'рядов')) : (r === 1 ? 'row' : 'rows') }}</span>
										<span class="text-[10px]" :class="tempRows === r ? 'text-purple-300' : 'text-secondary'">
											{{ r === 1 ? (isRu ? 'в одну строку' : 'single line') : (isRu ? `сетка до ${r}` : `grid up to ${r}`) }}
										</span>
									</button>
								</div>
							</div>

							<div class="p-4 rounded-2xl bg-surface-2 border border-divider flex flex-col gap-3">
								<h4 class="m-0 text-xs font-bold text-contrast">
									{{ isRu ? 'Фиксируемая активность' : 'Tracked activity' }}
								</h4>

								<div class="flex items-center justify-between">
									<div class="flex items-center gap-2 text-xs font-semibold text-contrast">
										<Toggle id="home-multiplayer-toggle" v-model="tempMultiplayer" />
										<span>{{ isRu ? 'В сетевых играх' : 'In multiplayer' }}</span>
									</div>
								</div>

								<div class="flex items-center justify-between">
									<div class="flex items-center gap-2 text-xs font-semibold text-contrast">
										<Toggle id="home-singleplayer-toggle" v-model="tempSingleplayer" />
										<span>{{ isRu ? 'В одиночных мирах' : 'In singleplayer' }}</span>
									</div>
								</div>
							</div>

							<div v-if="editingGroup.id.startsWith('custom_') || editingGroup.id === 'pinned'" class="p-4 rounded-2xl bg-surface-2 border border-divider flex flex-col gap-3">
								<h4 class="m-0 text-xs font-bold text-contrast">
									{{ isRu ? 'Содержимое группы' : 'Group content' }}
								</h4>

								<div class="grid grid-cols-3 gap-2">
									<button
										type="button"
										class="py-2 px-3 rounded-xl text-xs font-semibold border-2 cursor-pointer transition-all"
										:class="tempFilterType === 'all' ? 'bg-brand/15 text-white border-brand font-bold' : 'bg-surface-1 text-secondary border-white/10 hover:bg-surface-3'"
										@click="tempFilterType = 'all'"
									>
										{{ isRu ? 'Все сборки' : 'All instances' }}
									</button>
									<button
										type="button"
										class="py-2 px-3 rounded-xl text-xs font-semibold border-2 cursor-pointer transition-all"
										:class="tempFilterType === 'manual' ? 'bg-brand/15 text-white border-brand font-bold' : 'bg-surface-1 text-secondary border-white/10 hover:bg-surface-3'"
										@click="tempFilterType = 'manual'"
									>
										{{ isRu ? 'Вручную' : 'Manually' }}
									</button>
									<button
										type="button"
										class="py-2 px-3 rounded-xl text-xs font-semibold border-2 cursor-pointer transition-all"
										:class="tempFilterType === 'loader' ? 'bg-brand/15 text-white border-brand font-bold' : 'bg-surface-1 text-secondary border-white/10 hover:bg-surface-3'"
										@click="tempFilterType = 'loader'"
									>
										{{ isRu ? 'Загрузчик' : 'Loader' }}
									</button>
								</div>

								<div v-if="tempFilterType === 'manual'" class="flex flex-col gap-1.5 max-h-44 overflow-y-auto pr-1">
									<div
										v-for="inst in instances"
										:key="inst.id"
										class="flex items-center justify-between p-2.5 rounded-xl bg-surface-1 border border-divider hover:bg-surface-3 transition-colors cursor-pointer"
										@click="toggleManualSelection(inst.id, tempSelectedIds)"
									>
										<div class="flex items-center gap-2">
											<span class="text-xs font-medium text-contrast">{{ inst.name }}</span>
											<span class="text-[10px] text-secondary capitalize">({{ inst.loader }} {{ inst.game_version }})</span>
										</div>
										<div
											class="w-4 h-4 rounded border flex items-center justify-center transition-colors"
											:class="tempSelectedIds.includes(inst.id) ? 'bg-brand border-brand text-white' : 'border-divider bg-transparent'"
										>
											<svg v-if="tempSelectedIds.includes(inst.id)" class="w-3 h-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3">
												<polyline points="20 6 9 17 4 12"></polyline>
											</svg>
										</div>
									</div>
								</div>

								<div v-if="tempFilterType === 'loader'" class="flex gap-2">
									<button
										v-for="l in ['fabric', 'forge', 'neoforge', 'vanilla']"
										:key="l"
										type="button"
										class="flex-1 py-1.5 rounded-xl text-xs font-semibold capitalize border-2 cursor-pointer transition-all"
										:class="tempTargetLoader === l ? 'bg-brand/15 text-white border-brand font-bold' : 'bg-surface-1 text-secondary border-white/10 hover:bg-surface-3'"
										@click="tempTargetLoader = l"
									>
										{{ l }}
									</button>
								</div>
							</div>
						</div>

						<div class="flex items-center justify-end gap-3 pt-2 border-t border-divider">
							<button
								type="button"
								class="px-4 py-2.5 rounded-xl bg-transparent hover:bg-surface-3 text-secondary hover:text-contrast text-xs font-semibold border-0 cursor-pointer transition-colors"
								@click="closeGroupConfigModal"
							>
								{{ isRu ? 'Отмена' : 'Cancel' }}
							</button>
							<button
								type="button"
								class="px-6 py-2.5 rounded-xl bg-brand hover:brightness-110 active:scale-95 text-white font-bold text-xs uppercase tracking-wider border-0 cursor-pointer shadow-lg transition-all"
								@click="saveGroupConfig"
							>
								{{ isRu ? 'Сохранить' : 'Save' }}
							</button>
						</div>
					</div>
				</div>
			</Transition>
		</Teleport>

		<Teleport to="body">
			<Transition name="fade">
				<div
					v-if="showCreateGroupModal"
					class="fixed inset-0 z-[200] flex items-center justify-center bg-black/75 backdrop-blur-md p-4 overflow-y-auto select-none"
					@click.self="showCreateGroupModal = false"
				>
					<div
						class="w-full max-w-md bg-bg-raised border border-white/10 rounded-3xl p-6 shadow-2xl flex flex-col gap-5 text-contrast my-auto"
						@click.stop
					>
						<div class="flex items-center justify-between pb-3 border-b border-divider">
							<div class="flex items-center gap-3">
								<div class="w-9 h-9 rounded-2xl bg-brand/15 text-brand flex items-center justify-center border border-brand/20">
									<svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
										<line x1="12" y1="5" x2="12" y2="19"></line>
										<line x1="5" y1="12" x2="19" y2="12"></line>
									</svg>
								</div>
								<div>
									<h3 class="text-base font-bold text-contrast m-0">
										{{ isRu ? 'Создать новую группу' : 'Create new group' }}
									</h3>
									<p class="text-xs text-secondary m-0">
										{{ isRu ? 'Группа появится на вашей главной странице' : 'The group will appear on your home page' }}
									</p>
								</div>
							</div>

							<button
								class="w-8 h-8 rounded-xl bg-transparent hover:bg-surface-3 text-secondary hover:text-contrast flex items-center justify-center transition-colors border-0 cursor-pointer"
								:title="isRu ? 'Закрыть (Esc)' : 'Close (Esc)'"
								@click="showCreateGroupModal = false"
							>
								<svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
									<line x1="18" y1="6" x2="6" y2="18"></line>
									<line x1="6" y1="6" x2="18" y2="18"></line>
								</svg>
							</button>
						</div>

						<div class="flex flex-col gap-4">
							<div class="flex flex-col gap-1.5">
								<label class="text-xs font-semibold text-secondary">
									{{ isRu ? 'Название группы' : 'Group title' }}
								</label>
								<input
									v-model="newGroupTitle"
									type="text"
									:placeholder="isRu ? 'Например: Избранные сборки' : 'e.g. Favorite Instances'"
									class="w-full bg-surface-1 border border-divider focus:border-brand rounded-xl px-3.5 py-2.5 text-sm text-contrast placeholder:text-secondary/60 outline-none transition-colors"
								/>
							</div>

							<div class="p-4 rounded-2xl bg-surface-2 border border-divider flex flex-col gap-3">
								<label class="text-xs font-semibold text-contrast">{{ isRu ? 'Что будет в группе' : 'Group content' }}</label>
								<div class="grid grid-cols-3 gap-2">
									<button
										type="button"
										class="py-2 px-2 rounded-xl text-xs font-semibold border-2 cursor-pointer transition-all"
										:class="newGroupFilterType === 'all' ? 'bg-brand/15 text-white border-brand font-bold' : 'bg-surface-1 text-secondary border-white/10 hover:bg-surface-3'"
										@click="newGroupFilterType = 'all'"
									>
										{{ isRu ? 'Все сборки' : 'All instances' }}
									</button>
									<button
										type="button"
										class="py-2 px-2 rounded-xl text-xs font-semibold border-2 cursor-pointer transition-all"
										:class="newGroupFilterType === 'manual' ? 'bg-brand/15 text-white border-brand font-bold' : 'bg-surface-1 text-secondary border-white/10 hover:bg-surface-3'"
										@click="newGroupFilterType = 'manual'"
									>
										{{ isRu ? 'Вручную' : 'Manually' }}
									</button>
									<button
										type="button"
										class="py-2 px-2 rounded-xl text-xs font-semibold border-2 cursor-pointer transition-all"
										:class="newGroupFilterType === 'loader' ? 'bg-brand/15 text-white border-brand font-bold' : 'bg-surface-1 text-secondary border-white/10 hover:bg-surface-3'"
										@click="newGroupFilterType = 'loader'"
									>
										{{ isRu ? 'Загрузчик' : 'Loader' }}
									</button>
								</div>

								<div v-if="newGroupFilterType === 'manual'" class="flex flex-col gap-1.5 max-h-40 overflow-y-auto pr-1 mt-1">
									<div
										v-for="inst in instances"
										:key="inst.id"
										class="flex items-center justify-between p-2 rounded-xl bg-surface-1 border border-divider hover:bg-surface-3 transition-colors cursor-pointer"
										@click="toggleManualSelection(inst.id, newGroupSelectedIds)"
									>
										<div class="flex items-center gap-2">
											<span class="text-xs font-medium text-contrast">{{ inst.name }}</span>
											<span class="text-[10px] text-secondary capitalize">({{ inst.loader }} {{ inst.game_version }})</span>
										</div>
										<div
											class="w-4 h-4 rounded border flex items-center justify-center transition-colors"
											:class="newGroupSelectedIds.includes(inst.id) ? 'bg-brand border-brand text-white' : 'border-divider bg-transparent'"
										>
											<svg v-if="newGroupSelectedIds.includes(inst.id)" class="w-3 h-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3">
												<polyline points="20 6 9 17 4 12"></polyline>
											</svg>
										</div>
									</div>
								</div>

								<div v-if="newGroupFilterType === 'loader'" class="flex gap-1.5 mt-1">
									<button
										v-for="l in ['fabric', 'forge', 'neoforge', 'vanilla']"
										:key="l"
										type="button"
										class="flex-1 py-1.5 rounded-xl text-xs font-semibold capitalize border cursor-pointer transition-all"
										:class="newGroupTargetLoader === l ? 'bg-purple-500/15 text-white border-2 border-purple-500 font-bold' : 'bg-surface-1 text-secondary border border-white/10 hover:bg-surface-3'"
										@click="newGroupTargetLoader = l"
									>
										{{ l }}
									</button>
								</div>
							</div>
						</div>

						<div class="flex items-center justify-end gap-3 pt-2 border-t border-divider">
							<button
								type="button"
								class="px-4 py-2.5 rounded-xl bg-transparent hover:bg-surface-3 text-secondary hover:text-contrast text-xs font-semibold border-0 cursor-pointer transition-colors"
								@click="showCreateGroupModal = false"
							>
								{{ isRu ? 'Отмена' : 'Cancel' }}
							</button>
							<button
								type="button"
								class="px-6 py-2.5 rounded-xl bg-brand hover:brightness-110 active:scale-95 text-white font-bold text-xs uppercase tracking-wider border-0 cursor-pointer shadow-lg transition-all disabled:opacity-50 disabled:cursor-not-allowed"
								:disabled="!newGroupTitle.trim()"
								@click="handleCreateNewGroup"
							>
								{{ isRu ? 'Создать' : 'Create' }}
							</button>
						</div>
					</div>
				</div>
			</Transition>
		</Teleport>

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
.fade-enter-active,
.fade-leave-active {
	transition: opacity 0.2s ease, transform 0.2s ease;
}
.fade-enter-from,
.fade-leave-to {
	opacity: 0;
	transform: translate(-50%, 10px);
}
</style>
