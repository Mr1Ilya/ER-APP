<script setup lang="ts">
import {
	CheckIcon,
	DownloadIcon,
	ExternalIcon,
	PlusIcon,
	RefreshCwIcon,
	SpinnerIcon,
	XIcon,
} from '@erteam/assets'
import { ButtonStyled, NewModal } from '@erteam/ui'
import { openUrl } from '@tauri-apps/plugin-opener'
import { computed, ref } from 'vue'
import { useRouter } from 'vue-router'

import {
	getCurseForgeDescription,
	getCurseForgeFileDownloadUrl,
	getCurseForgeModpackFiles,
	installCurseForgeModpack,
	type CurseForgeFileItem,
} from '@/helpers/curseforge'
import i18n from '@/i18n.config'

const router = useRouter()
const isRu = computed(() => (i18n.global.locale.value || '').startsWith('ru'))

const modal = ref<InstanceType<typeof NewModal>>()
const project = ref<any>(null)
const descriptionHtml = ref<string>('')
const loadingDesc = ref(false)
const files = ref<CurseForgeFileItem[]>([])
const loadingFiles = ref(false)
const activeTab = ref<'description' | 'versions'>('description')

const isInstalling = ref(false)
const installProgressText = ref('')
const installProgressPercent = ref<number | null>(null)
const installError = ref('')

async function open(proj: any) {
	project.value = proj
	activeTab.value = 'description'
	descriptionHtml.value = ''
	files.value = []
	isInstalling.value = false
	installProgressText.value = ''
	installProgressPercent.value = null
	installError.value = ''
	modal.value?.show()

	loadingDesc.value = true
	loadingFiles.value = true

	getCurseForgeDescription(proj.cf_id)
		.then((html) => {
			descriptionHtml.value = html || (isRu.value ? 'Описание отсутствует' : 'No description provided')
		})
		.catch(() => {
			descriptionHtml.value = isRu.value ? 'Не удалось загрузить описание' : 'Failed to load description'
		})
		.finally(() => {
			loadingDesc.value = false
		})

	getCurseForgeModpackFiles(proj.cf_id)
		.then((res) => {
			files.value = res || []
		})
		.catch(() => {
			files.value = []
		})
		.finally(() => {
			loadingFiles.value = false
		})
}

function formatBytes(bytes?: number): string {
	if (!bytes) return ''
	const mb = bytes / (1024 * 1024)
	return `${mb.toFixed(1)} MB`
}

function formatDate(dateStr?: string): string {
	if (!dateStr) return ''
	try {
		return new Date(dateStr).toLocaleDateString()
	} catch {
		return dateStr
	}
}

async function handleInstall(file?: CurseForgeFileItem) {
	if (isInstalling.value || !project.value) return
	isInstalling.value = true
	installError.value = ''
	installProgressText.value = isRu.value ? 'Подготовка к установке...' : 'Preparing installation...'

	try {
		const targetFile = file || files.value[0]
		const downloadUrl = targetFile ? getCurseForgeFileDownloadUrl(targetFile) : undefined

		await installCurseForgeModpack({
			modId: project.value.cf_id,
			fileId: targetFile?.id,
			downloadUrl,
			customName: project.value.title,
			iconUrl: project.value.icon_url,
			onProgress: (status, current, total) => {
				installProgressText.value = status
				if (current !== undefined && total !== undefined && total > 0) {
					installProgressPercent.value = Math.round((current / total) * 100)
				}
			},
		})

		modal.value?.hide()
		await router.push('/library')
	} catch (err: any) {
		installError.value = err?.message || String(err)
	} finally {
		isInstalling.value = false
	}
}

function openCurseForgePage() {
	if (!project.value?.slug) return
	openUrl(`https://www.curseforge.com/minecraft/modpacks/${project.value.slug}`)
}

function openAuthorUrl(url?: string) {
	if (!url) return
	openUrl(url)
}

defineExpose({
	open,
	hide: () => modal.value?.hide(),
})
</script>

<template>
	<NewModal ref="modal" :closable="!isInstalling" max-width="840px">
		<template #title>
			<div v-if="project" class="flex items-start gap-4">
				<img
					v-if="project.icon_url"
					:src="project.icon_url"
					alt="Pack logo"
					class="w-14 h-14 rounded-2xl object-cover border border-solid border-white/10 shrink-0 bg-surface-3"
				/>
				<div class="flex flex-col gap-1 min-w-0">
					<div class="flex items-center gap-2 flex-wrap">
						<h2 class="text-xl font-bold text-white tracking-wide m-0 line-clamp-1">
							{{ project.title }}
						</h2>
						<span
							class="inline-flex items-center gap-1 px-2 py-0.5 rounded-md text-[11px] font-semibold bg-orange-500/15 text-orange-400 border border-solid border-orange-500/30 shrink-0"
						>
							CurseForge
						</span>
					</div>
					<div class="flex items-center gap-3 text-xs text-secondary flex-wrap">
						<span v-if="project.author" class="flex items-center gap-1">
							<span>by</span>
							<a
								v-if="project.author_url"
								href="javascript:void(0)"
								class="text-contrast hover:text-purple-400 hover:underline font-semibold cursor-pointer inline-flex items-center gap-1"
								@click="openAuthorUrl(project.author_url)"
							>
								<span>{{ project.author }}</span>
								<ExternalIcon class="w-3 h-3 text-secondary" />
							</a>
							<strong v-else class="text-contrast">{{ project.author }}</strong>
						</span>
						<span v-if="project.downloads" class="flex items-center gap-1">
							<DownloadIcon class="w-3.5 h-3.5" />
							{{ (project.downloads / 1000000).toFixed(2) }}M
						</span>
					</div>
				</div>
			</div>
		</template>

		<div v-if="project" class="flex flex-col gap-4 text-contrast">
			<div v-if="project.description" class="px-3.5 py-2.5 rounded-xl bg-surface-2 border border-solid border-white/10 text-xs text-secondary leading-relaxed">
				{{ project.description }}
			</div>

			<div class="flex items-center justify-between border-b border-solid border-white/10 pb-2">
				<div class="flex items-center gap-2">
					<button
						type="button"
						class="px-4 py-1.5 rounded-xl text-xs font-semibold cursor-pointer transition-colors border-2 border-solid"
						:class="activeTab === 'description' ? 'border-2 border-solid border-purple-500 bg-[rgba(172,81,251,0.18)] text-white' : 'border-2 border-solid border-white/10 hover:border-white/20 text-secondary hover:text-contrast bg-surface-2'"
						@click="activeTab = 'description'"
					>
						{{ isRu ? 'Описание' : 'Description' }}
					</button>
					<button
						type="button"
						class="px-4 py-1.5 rounded-xl text-xs font-semibold cursor-pointer transition-colors border-2 border-solid flex items-center gap-1.5"
						:class="activeTab === 'versions' ? 'border-2 border-solid border-purple-500 bg-[rgba(172,81,251,0.18)] text-white' : 'border-2 border-solid border-white/10 hover:border-white/20 text-secondary hover:text-contrast bg-surface-2'"
						@click="activeTab = 'versions'"
					>
						<span>{{ isRu ? 'Версии' : 'Versions' }}</span>
						<span v-if="files.length" class="px-1.5 py-0.2 rounded-full bg-white/10 text-[10px]">{{ files.length }}</span>
					</button>
				</div>

				<button
					type="button"
					class="flex items-center gap-1.5 px-3 py-1.5 rounded-xl bg-surface-2 hover:bg-surface-3 text-secondary hover:text-contrast text-xs font-semibold border border-solid border-white/10 cursor-pointer transition-colors"
					@click="openCurseForgePage"
				>
					<span>{{ isRu ? 'Открыть на CurseForge' : 'View on CurseForge' }}</span>
					<ExternalIcon class="w-3.5 h-3.5" />
				</button>
			</div>

			<div v-if="activeTab === 'description'" class="max-h-[50vh] overflow-y-auto pr-2 custom-scrollbar flex flex-col gap-3">
				<div v-if="loadingDesc" class="flex flex-col items-center justify-center p-12 gap-3 text-secondary">
					<SpinnerIcon class="w-8 h-8 text-purple-400 animate-spin" />
					<span class="text-xs">{{ isRu ? 'Загрузка описания сборки...' : 'Loading pack description...' }}</span>
				</div>
				<div
					v-else
					class="prose prose-invert max-w-none text-sm text-secondary/90 leading-relaxed [&_h1]:text-white [&_h2]:text-white [&_h3]:text-white [&_h4]:text-white [&_a]:text-purple-400 [&_a]:underline [&_img]:max-w-full [&_img]:rounded-xl [&_img]:my-2 [&_p]:my-2 [&_ul]:list-disc [&_ul]:pl-5 [&_ol]:list-decimal [&_ol]:pl-5 [&_table]:w-full [&_table]:border-collapse [&_th]:border [&_th]:border-white/10 [&_th]:p-2 [&_th]:bg-surface-2 [&_td]:border [&_td]:border-white/10 [&_td]:p-2"
					v-html="descriptionHtml"
				></div>
			</div>

			<div v-else-if="activeTab === 'versions'" class="max-h-[50vh] overflow-y-auto pr-2 custom-scrollbar flex flex-col gap-2">
				<div v-if="loadingFiles" class="flex flex-col items-center justify-center p-12 gap-3 text-secondary">
					<SpinnerIcon class="w-8 h-8 text-purple-400 animate-spin" />
					<span class="text-xs">{{ isRu ? 'Получение списка версий сборки...' : 'Loading versions list...' }}</span>
				</div>
				<div v-else-if="files.length === 0" class="p-8 text-center text-secondary text-xs">
					{{ isRu ? 'Файлы для загрузки не найдены' : 'No downloadable files found' }}
				</div>
				<div
					v-for="file in files"
					:key="file.id"
					class="p-3 rounded-2xl bg-surface-2 border border-solid border-white/10 hover:border-white/20 flex items-center justify-between gap-3 transition-colors"
				>
					<div class="flex flex-col gap-1 min-w-0">
						<span class="font-bold text-sm text-white line-clamp-1">
							{{ file.displayName || file.fileName }}
						</span>
						<div class="flex items-center gap-2 text-xs text-secondary flex-wrap">
							<span v-if="file.gameVersions?.length" class="px-2 py-0.5 rounded-md bg-white/5 border border-solid border-white/10 text-[11px] font-medium text-contrast">
								{{ file.gameVersions.join(', ') }}
							</span>
							<span v-if="file.fileLength">{{ formatBytes(file.fileLength) }}</span>
							<span v-if="file.fileDate">{{ formatDate(file.fileDate) }}</span>
						</div>
					</div>

					<button
						type="button"
						class="px-3.5 py-1.5 rounded-xl text-xs font-bold text-white bg-purple-600 hover:bg-purple-500 border border-solid border-purple-400/30 cursor-pointer transition-colors shrink-0 flex items-center gap-1.5 disabled:opacity-50 disabled:cursor-not-allowed"
						:disabled="isInstalling"
						@click="handleInstall(file)"
					>
						<DownloadIcon class="w-3.5 h-3.5" />
						<span>{{ isRu ? 'Установить' : 'Install' }}</span>
					</button>
				</div>
			</div>

			<div v-if="installError" class="p-3 rounded-xl bg-red-500/10 border border-solid border-red-500/30 text-red-400 text-xs flex items-center gap-2">
				<XIcon class="w-4 h-4 shrink-0" />
				<span>{{ installError }}</span>
			</div>

			<div v-if="isInstalling" class="p-3.5 rounded-2xl bg-surface-2 border border-solid border-purple-500/30 flex flex-col gap-2">
				<div class="flex items-center justify-between text-xs font-semibold text-contrast">
					<span class="flex items-center gap-2">
						<SpinnerIcon class="w-4 h-4 animate-spin text-purple-400" />
						{{ installProgressText }}
					</span>
					<span v-if="installProgressPercent !== null">{{ installProgressPercent }}%</span>
				</div>
				<div class="w-full h-1.5 rounded-full bg-white/10 overflow-hidden">
					<div
						class="h-full bg-purple-500 transition-all duration-300 rounded-full"
						:style="{ width: `${installProgressPercent ?? 100}%` }"
					></div>
				</div>
			</div>

			<div class="flex items-center justify-end gap-3 pt-3 border-t border-solid border-white/10">
				<button
					type="button"
					class="px-4 py-2 rounded-xl text-xs font-semibold text-secondary hover:text-contrast bg-surface-2 hover:bg-surface-3 border border-solid border-white/10 cursor-pointer transition-colors"
					:disabled="isInstalling"
					@click="modal?.hide()"
				>
					{{ isRu ? 'Закрыть' : 'Close' }}
				</button>

				<button
					type="button"
					class="px-5 py-2 rounded-xl text-xs font-bold text-white bg-purple-600 hover:bg-purple-500 border border-solid border-purple-400/30 cursor-pointer transition-colors flex items-center gap-2 disabled:opacity-50 disabled:cursor-not-allowed"
					:disabled="isInstalling"
					@click="handleInstall()"
				>
					<SpinnerIcon v-if="isInstalling" class="w-4 h-4 animate-spin" />
					<PlusIcon v-else class="w-4 h-4" />
					<span>{{ isInstalling ? (isRu ? 'Установка...' : 'Installing...') : (isRu ? 'Установить последнюю версию' : 'Install latest version') }}</span>
				</button>
			</div>
		</div>
	</NewModal>
</template>
