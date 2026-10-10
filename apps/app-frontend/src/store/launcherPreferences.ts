import { ref, watch } from 'vue'

export interface HomeGroupConfig {
	id: string
	title: string
	icon: string
	badge?: number
	locked?: boolean
	visible: boolean
	rows: 1 | 2 | 3 | 4
	activityMultiplayer: boolean
	activitySingleplayer: boolean
	filterType: 'all' | 'manual' | 'version' | 'loader'
	selectedInstanceIds: string[]
	targetVersion?: string
	targetLoader?: string
}

export interface ConnectionItem {
	id: string
	title: string
	subtitle: string
	host: string
	locked: boolean
	pingMs: number
	active: boolean
	type: 'direct' | 'system' | 'custom'
}

export interface ConnectionSettingsState {
	autoSelect: boolean
	items: ConnectionItem[]
}

const DEFAULT_HOME_GROUPS: HomeGroupConfig[] = [
	{
		id: 'news',
		title: 'Слайдер новостей',
		icon: 'news',
		locked: true,
		visible: true,
		rows: 1,
		activityMultiplayer: true,
		activitySingleplayer: true,
		filterType: 'all',
		selectedInstanceIds: [],
	},
	{
		id: 'recent',
		title: 'Недавние запуски',
		icon: 'zap',
		badge: 2,
		locked: false,
		visible: true,
		rows: 2,
		activityMultiplayer: true,
		activitySingleplayer: true,
		filterType: 'all',
		selectedInstanceIds: [],
	},
	{
		id: 'releases',
		title: 'Последние релизы',
		icon: 'creeper',
		badge: 2,
		locked: false,
		visible: true,
		rows: 2,
		activityMultiplayer: true,
		activitySingleplayer: true,
		filterType: 'all',
		selectedInstanceIds: [],
	},
	{
		id: 'pinned',
		title: 'Закрепленные',
		icon: 'pin',
		badge: 0,
		locked: false,
		visible: true,
		rows: 1,
		activityMultiplayer: true,
		activitySingleplayer: true,
		filterType: 'manual',
		selectedInstanceIds: [],
	},
]

const DEFAULT_CONNECTIONS: ConnectionSettingsState = {
	autoSelect: true,
	items: [
		{
			id: 'direct',
			title: 'Прямое подключение',
			subtitle: 'Не использовать прокси для соединений',
			host: 'https://end-rage.ru',
			locked: true,
			pingMs: 190,
			active: true,
			type: 'direct',
		},
		{
			id: 'system',
			title: 'Системный прокси',
			subtitle: 'Системный прокси отключен',
			host: 'https://api.end-rage.ru',
			locked: true,
			pingMs: 190,
			active: false,
			type: 'system',
		},
	],
}

function loadFromStorage<T>(key: string, fallback: T): T {
	if (typeof window === 'undefined') return fallback
	try {
		const raw = localStorage.getItem(key)
		if (!raw) return fallback
		return JSON.parse(raw) as T
	} catch {
		return fallback
	}
}

function saveToStorage<T>(key: string, value: T): void {
	if (typeof window === 'undefined') return
	try {
		localStorage.setItem(key, JSON.stringify(value))
	} catch {}
}

export const streamerMode = ref<boolean>(loadFromStorage('er_streamer_mode', false))

watch(
	streamerMode,
	(val) => {
		saveToStorage('er_streamer_mode', val)
		if (typeof window !== 'undefined') {
			window.dispatchEvent(new CustomEvent('er-streamer-mode-changed', { detail: val }))
		}
	},
	{ immediate: true },
)

export const pinnedInstanceIds = ref<string[]>(loadFromStorage('er_pinned_instances', []))

watch(
	pinnedInstanceIds,
	(val) => {
		saveToStorage('er_pinned_instances', val)
		if (typeof window !== 'undefined') {
			window.dispatchEvent(new CustomEvent('er-pinned-instances-changed', { detail: val }))
		}
	},
	{ deep: true },
)

export function isInstancePinned(instanceId: string): boolean {
	return pinnedInstanceIds.value.includes(instanceId)
}

export function togglePinInstance(instanceId: string): void {
	const idx = pinnedInstanceIds.value.indexOf(instanceId)
	if (idx === -1) {
		pinnedInstanceIds.value.push(instanceId)
	} else {
		pinnedInstanceIds.value.splice(idx, 1)
	}
	const pinnedGroup = homeGroups.value.find((g) => g.id === 'pinned')
	if (pinnedGroup) {
		pinnedGroup.badge = pinnedInstanceIds.value.length
	}
}

export const homeGroups = ref<HomeGroupConfig[]>(loadFromStorage('er_home_groups', DEFAULT_HOME_GROUPS))

watch(
	homeGroups,
	(val) => {
		saveToStorage('er_home_groups', val)
		if (typeof window !== 'undefined') {
			window.dispatchEvent(new CustomEvent('er-home-groups-changed', { detail: val }))
		}
	},
	{ deep: true },
)

export const connectionSettings = ref<ConnectionSettingsState>(
	loadFromStorage('er_connection_settings', DEFAULT_CONNECTIONS),
)

watch(
	connectionSettings,
	(val) => {
		saveToStorage('er_connection_settings', val)
	},
	{ deep: true },
)

export function updateHomeGroup(id: string, updates: Partial<HomeGroupConfig>): void {
	const index = homeGroups.value.findIndex((g) => g.id === id)
	if (index !== -1) {
		homeGroups.value[index] = { ...homeGroups.value[index], ...updates }
	}
}

export function toggleHomeGroupVisibility(id: string): void {
	const group = homeGroups.value.find((g) => g.id === id)
	if (group && !group.locked) {
		group.visible = !group.visible
	}
}

export function moveHomeGroup(fromIndex: number, toIndex: number): void {
	if (fromIndex <= 0 || toIndex <= 0) return
	if (toIndex >= homeGroups.value.length) return
	const item = homeGroups.value.splice(fromIndex, 1)[0]
	homeGroups.value.splice(toIndex, 0, item)
}

export function deleteHomeGroup(id: string): void {
	const index = homeGroups.value.findIndex((g) => g.id === id)
	if (index !== -1 && !homeGroups.value[index].locked) {
		homeGroups.value.splice(index, 1)
	}
}

export function addCustomHomeGroup(params: {
	title: string
	filterType: 'all' | 'manual' | 'version' | 'loader'
	selectedInstanceIds: string[]
	targetVersion?: string
	targetLoader?: string
}): void {
	const newId = 'custom_' + Date.now()
	homeGroups.value.push({
		id: newId,
		title: params.title.trim() || 'Новая группа',
		icon: 'folder',
		badge: params.selectedInstanceIds.length,
		locked: false,
		visible: true,
		rows: 1,
		activityMultiplayer: true,
		activitySingleplayer: true,
		filterType: params.filterType,
		selectedInstanceIds: params.selectedInstanceIds,
		targetVersion: params.targetVersion,
		targetLoader: params.targetLoader,
	})
}

export async function measureRealPing(targetHost: string): Promise<number> {
	let url = targetHost.trim()
	if (!url.startsWith('http://') && !url.startsWith('https://')) {
		url = `https://${url}`
	}
	const start = performance.now()
	try {
		const controller = new AbortController()
		const timer = setTimeout(() => controller.abort(), 4000)
		await fetch(url, {
			method: 'HEAD',
			mode: 'no-cors',
			cache: 'no-cache',
			signal: controller.signal,
		})
		clearTimeout(timer)
		return Math.max(1, Math.round(performance.now() - start))
	} catch {
		const elapsed = Math.round(performance.now() - start)
		return Math.max(1, Math.min(elapsed, 450))
	}
}

export async function refreshAllPings(): Promise<void> {
	for (const item of connectionSettings.value.items) {
		try {
			item.pingMs = await measureRealPing(item.host || 'https://end-rage.ru')
		} catch {
			item.pingMs = 190
		}
	}
}
