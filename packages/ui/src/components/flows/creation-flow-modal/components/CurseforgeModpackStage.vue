<template>
	<div class="flex flex-col gap-4">
		<!-- Active installation progress overlay/state -->
		<template v-if="installing">
			<div class="flex flex-col items-center justify-center gap-4 py-8 text-center">
				<SpinnerIcon class="size-10 animate-spin text-brand" />
				<div class="flex flex-col gap-1">
					<span class="text-base font-semibold text-contrast">
						{{ currentProgressStatus || formatMessage(messages.installingPrompt) }}
					</span>
					<span v-if="progressPercent !== null" class="text-sm text-secondary">
						{{ progressPercent }}%
					</span>
				</div>
				<div
					v-if="progressPercent !== null"
					class="h-2 w-full overflow-hidden rounded-full bg-surface-4"
				>
					<div
						class="h-full bg-brand transition-all duration-300"
						:style="{ width: `${progressPercent}%` }"
					/>
				</div>
			</div>
		</template>

		<!-- Main Selection Flow -->
		<template v-else>
			<span class="font-semibold text-contrast">
				{{ formatMessage(messages.knownCurseforgeModpackPrompt) }}
			</span>

			<!-- Search Combobox -->
			<Combobox
				v-model="selectedModId"
				:options="searchOptions"
				searchable
				:search-placeholder="formatMessage(messages.searchPlaceholder)"
				:no-options-message="
					searchLoading
						? formatMessage(commonMessages.loadingLabel)
						: formatMessage(messages.noResultsFound)
				"
				:disable-search-filter="true"
				@search-input="handleSearch"
			>
				<template #option-suffix>
					<RightArrowIcon
						class="size-5 shrink-0 text-secondary opacity-0 transition-opacity group-hover/option:opacity-100 group-data-[focused=true]/option:opacity-100"
					/>
				</template>
			</Combobox>

			<!-- Selected Modpack Details Card -->
			<div
				v-if="selectedMod"
				class="flex flex-col gap-3 rounded-2xl border border-surface-5 bg-surface-2 p-4"
			>
				<div class="flex items-center gap-3">
					<img
						v-if="selectedMod.logo?.thumbnailUrl"
						:src="selectedMod.logo.thumbnailUrl"
						:alt="selectedMod.name"
						class="size-12 shrink-0 rounded-xl object-cover"
					/>
					<div class="flex flex-1 flex-col overflow-hidden">
						<span class="truncate font-semibold text-contrast">{{ selectedMod.name }}</span>
						<span class="line-clamp-1 text-xs text-secondary">{{ selectedMod.summary }}</span>
					</div>
				</div>

				<!-- Version selection if files loaded -->
				<div v-if="filesLoading" class="flex items-center gap-2 text-sm text-secondary">
					<SpinnerIcon class="size-4 animate-spin text-brand" />
					{{ formatMessage(commonMessages.loadingLabel) }}
				</div>
				<div v-else-if="fileOptions.length > 0" class="flex flex-col gap-2">
					<span class="text-xs font-semibold text-secondary">
						{{ formatMessage(messages.selectVersion) }}
					</span>
					<select
						v-model="selectedFileId"
						class="w-full rounded-xl border border-surface-5 bg-surface-3 px-3 py-2 text-sm text-contrast focus:border-brand focus:outline-none"
					>
						<option v-for="opt in fileOptions" :key="opt.value" :value="opt.value">
							{{ opt.label }}
						</option>
					</select>
				</div>

				<ButtonStyled color="brand" class="mt-1">
					<button
						type="button"
						class="flex w-full items-center justify-center gap-2"
						:disabled="!selectedFileId || installing"
						@click="installSelectedModpack"
					>
						<DownloadIcon class="size-5" />
						{{ formatMessage(messages.installModpackButton) }}
					</button>
				</ButtonStyled>
			</div>

			<div class="flex items-center gap-3">
				<div class="h-[1px] w-full flex-1 bg-surface-5" />
				<span class="text-sm text-secondary">{{ formatMessage(commonMessages.orLabel) }}</span>
				<div class="h-[1px] w-full flex-1 bg-surface-5" />
			</div>

			<!-- Alternative Actions -->
			<div class="flex flex-col gap-3">
				<div class="flex gap-3">
					<ButtonStyled type="outlined" class="flex-1">
						<button
							type="button"
							class="flex w-full items-center justify-center gap-2"
							:disabled="installing"
							@click="triggerZipFileInput"
						>
							<ImportIcon class="size-5" />
							{{ formatMessage(messages.importFromZip) }}
						</button>
					</ButtonStyled>

					<ButtonStyled type="outlined" class="flex-1">
						<button
							type="button"
							class="flex w-full items-center justify-center gap-2"
							:disabled="installing"
							@click="showUrlInput = !showUrlInput"
						>
							<LinkIcon class="size-5" />
							{{ formatMessage(messages.installFromUrl) }}
						</button>
					</ButtonStyled>
				</div>

				<!-- URL Input field -->
				<div v-if="showUrlInput" class="flex flex-col gap-2 rounded-xl bg-surface-2 p-3">
					<div class="flex gap-2">
						<input
							v-model="curseforgeUrl"
							type="url"
							placeholder="https://www.curseforge.com/minecraft/modpacks/.../files/..."
							class="flex-1 rounded-lg border border-surface-5 bg-surface-3 px-3 py-2 text-sm text-contrast focus:border-brand focus:outline-none"
							@keyup.enter="installFromUrl"
						/>
						<ButtonStyled color="brand">
							<button
								type="button"
								class="px-4"
								:disabled="!curseforgeUrl.trim() || installing"
								@click="installFromUrl"
							>
								{{ formatMessage(messages.installButton) }}
							</button>
						</ButtonStyled>
					</div>
				</div>
			</div>
		</template>
	</div>
</template>

<script setup lang="ts">
import {
	DownloadIcon,
	ImportIcon,
	LinkIcon,
	RightArrowIcon,
	SpinnerIcon,
} from '@erteam/assets'
import { commonMessages, defineMessages, useVIntl } from '@erteam/ui'
import { defineAsyncComponent, h, onMounted, ref, watch } from 'vue'

import ButtonStyled from '../../../base/ButtonStyled.vue'
import Combobox from '../../../base/Combobox.vue'
import { injectFilePicker } from '../../../../providers'
import { injectCreationFlowContext } from '../creation-flow-context'

const ctx = injectCreationFlowContext()
const filePicker = injectFilePicker()
const { formatMessage } = useVIntl()

const messages = defineMessages({
	knownCurseforgeModpackPrompt: {
		id: 'creation-flow.modal.curseforge.known-prompt',
		defaultMessage: 'Already know the CurseForge modpack you want to install?',
	},
	searchPlaceholder: {
		id: 'creation-flow.modal.curseforge.search-placeholder',
		defaultMessage: 'Search CurseForge modpacks...',
	},
	noResultsFound: {
		id: 'creation-flow.modal.curseforge.no-results',
		defaultMessage: 'No modpacks found',
	},
	selectVersion: {
		id: 'creation-flow.modal.curseforge.select-version',
		defaultMessage: 'Version to install:',
	},
	installModpackButton: {
		id: 'creation-flow.modal.curseforge.install-button',
		defaultMessage: 'Install modpack',
	},
	importFromZip: {
		id: 'creation-flow.modal.curseforge.import-zip',
		defaultMessage: 'Import from .zip file',
	},
	installFromUrl: {
		id: 'creation-flow.modal.curseforge.install-url',
		defaultMessage: 'Install by link',
	},
	installButton: {
		id: 'creation-flow.modal.curseforge.submit-install',
		defaultMessage: 'Install',
	},
	installingPrompt: {
		id: 'creation-flow.modal.curseforge.installing-prompt',
		defaultMessage: 'Installing CurseForge modpack...',
	},
})

const CF_KEY = '$2a$10$bL4bIL5pUWqfcO7KQtnMReakwtfHbNKh6v1uTpKlzhwoueEJQnPnm'

interface CFModItem {
	id: number
	name: string
	summary: string
	logo?: { thumbnailUrl: string }
	latestFilesIndexes?: { gameVersion: string; fileId: number; filename: string }[]
}

interface CFFileItem {
	id: number
	displayName: string
	fileName: string
	downloadUrl: string | null
	gameVersions: string[]
}

const searchLoading = ref(false)
const searchOptions = ref<{ label: string; value: string; icon?: any }[]>([])
const searchHits = ref<Record<string, CFModItem>>({})
const selectedModId = ref<string | undefined>(undefined)
const selectedMod = ref<CFModItem | null>(null)

const filesLoading = ref(false)
const fileOptions = ref<{ label: string; value: number; downloadUrl: string | null; fileName: string }[]>([])
const selectedFileId = ref<number | undefined>(undefined)

const showUrlInput = ref(false)
const curseforgeUrl = ref('')

const installing = ref(false)
const currentProgressStatus = ref('')
const progressPercent = ref<number | null>(null)

async function cfApi<T>(path: string, options: RequestInit = {}): Promise<T> {
	const res = await fetch(`https://api.curseforge.com/v1${path}`, {
		...options,
		headers: {
			'x-api-key': CF_KEY,
			'Accept': 'application/json',
			...(options.headers || {}),
		},
	})
	if (!res.ok) throw new Error(`CurseForge API error: ${res.status}`)
	return (await res.json()) as T
}

const search = async (query: string) => {
	searchLoading.value = true
	try {
		const searchFilter = query.trim() ? `&searchFilter=${encodeURIComponent(query.trim())}` : ''
		const res = await cfApi<{ data: CFModItem[] }>(
			`/mods/search?gameId=432&classId=4471&pageSize=15&sortField=2&sortOrder=desc${searchFilter}`,
		)
		const hits = res.data || []
		searchHits.value = {}
		for (const hit of hits) {
			searchHits.value[String(hit.id)] = hit
		}

		searchOptions.value = hits.map((hit) => ({
			label: hit.name,
			value: String(hit.id),
			icon: hit.logo?.thumbnailUrl
				? defineAsyncComponent(() =>
						Promise.resolve({
							setup: () => () =>
								h('img', {
									src: hit.logo?.thumbnailUrl,
									alt: hit.name,
									class: 'size-5 rounded object-cover',
								}),
						}),
				  )
				: undefined,
		}))
	} catch (err) {
		console.error('CurseForge search error:', err)
		searchOptions.value = []
	} finally {
		searchLoading.value = false
	}
}

let searchTimeout: any = null
const handleSearch = (query: string) => {
	clearTimeout(searchTimeout)
	searchTimeout = setTimeout(() => {
		search(query)
	}, 300)
}

watch(selectedModId, async (newId) => {
	if (!newId) {
		selectedMod.value = null
		fileOptions.value = []
		selectedFileId.value = undefined
		return
	}

	const mod = searchHits.value[newId]
	selectedMod.value = mod ?? null
	if (!mod) return

	filesLoading.value = true
	fileOptions.value = []
	selectedFileId.value = undefined

	try {
		const filesRes = await cfApi<{ data: CFFileItem[] }>(`/mods/${mod.id}/files?pageSize=15`)
		const files = filesRes.data || []
		fileOptions.value = files.map((f) => {
			const gv = f.gameVersions?.filter((v) => /^\d+\.\d+/.test(v) || ['Forge', 'Fabric', 'NeoForge'].includes(v)).join(' ') || ''
			return {
				label: `${f.displayName || f.fileName}${gv ? ` (${gv})` : ''}`,
				value: f.id,
				downloadUrl: f.downloadUrl,
				fileName: f.fileName,
			}
		})
		if (fileOptions.value.length > 0) {
			selectedFileId.value = fileOptions.value[0].value
		}
	} catch (err) {
		console.error('Failed to load files:', err)
	} finally {
		filesLoading.value = false
	}
})

async function installSelectedModpack() {
	if (!selectedMod.value || !selectedFileId.value) return
	const fileOpt = fileOptions.value.find((f) => f.value === selectedFileId.value)
	if (!fileOpt) return

	const downloadUrl =
		fileOpt.downloadUrl ||
		`https://edge.forgecdn.net/files/${Math.floor(fileOpt.value / 1000)}/${fileOpt.value % 1000}/${encodeURIComponent(fileOpt.fileName)}`

	await runInstallation({
		downloadUrl,
		customName: selectedMod.value.name,
	})
}

async function triggerZipFileInput() {
	try {
		if (filePicker?.pickModpackFile) {
			const picked = await filePicker.pickModpackFile()
			if (!picked) return
			if (picked.path) {
				await runInstallation({ filePath: picked.path })
				return
			}
			if (picked.file) {
				await runInstallation({ zipFile: picked.file })
				return
			}
		}

		// Browser input fallback
		const input = document.createElement('input')
		input.type = 'file'
		input.accept = '.zip'
		input.onchange = async (e: any) => {
			const file = e.target.files?.[0]
			if (file) {
				await runInstallation({ zipFile: file })
			}
		}
		input.click()
	} catch (err: any) {
		alert(`Ошибка при выборе файла: ${err.message || err}`)
	}
}

async function installFromUrl() {
	const url = curseforgeUrl.value.trim()
	if (!url) return

	// If direct zip link
	if (url.endsWith('.zip')) {
		await runInstallation({ downloadUrl: url })
		return
	}

	// If CurseForge files URL, e.g. .../files/123456
	const fileMatch = url.match(/\/files\/(\d+)/)
	if (fileMatch) {
		const fileId = parseInt(fileMatch[1], 10)
		try {
			installing.value = true
			currentProgressStatus.value = 'Получение данных о файле CurseForge...'
			const res = await cfApi<{ data: CFFileItem[] }>('/mods/files', {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({ fileIds: [fileId] }),
			})
			const file = res.data?.[0]
			if (file) {
				const downloadUrl =
					file.downloadUrl ||
					`https://edge.forgecdn.net/files/${Math.floor(file.id / 1000)}/${file.id % 1000}/${encodeURIComponent(file.fileName)}`
				await runInstallation({ downloadUrl })
				return
			}
		} catch (err: any) {
			alert(`Не удалось найти файл по ссылке: ${err.message}`)
			installing.value = false
			return
		}
	}

	alert('Пожалуйста, введите прямую ссылку на файл версии сборки (содержащую /files/<ID>).')
}

async function runInstallation(options: {
	filePath?: string
	zipData?: Uint8Array | ArrayBuffer
	zipFile?: File
	downloadUrl?: string
	customName?: string
}) {
	installing.value = true
	progressPercent.value = null
	currentProgressStatus.value = 'Подготовка к установке...'

	try {
		const installFn = (window as any).__installCurseForgeModpack
		if (!installFn) {
			throw new Error('Функция установки CurseForge недоступна')
		}
		await installFn({
			...options,
			onProgress: (status: string, current?: number, total?: number) => {
				currentProgressStatus.value = status
				if (typeof current === 'number' && typeof total === 'number' && total > 0) {
					progressPercent.value = Math.round((current / total) * 100)
				}
			},
		})

		currentProgressStatus.value = 'Сборка успешно установлена!'
		setTimeout(() => {
			ctx.modal.value?.hide()
			// Reload page or list
			window.location.reload()
		}, 1200)
	} catch (err: any) {
		console.error('CurseForge install error:', err)
		alert(`Ошибка установки сборки: ${err.message || err}`)
		installing.value = false
	}
}

onMounted(() => {
	search('')
})
</script>
