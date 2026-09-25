<script setup>
import { BoxIcon, FolderOpenIcon, FolderSearchIcon, TrashIcon } from '@erteam/assets'
import { ButtonStyled, injectNotificationManager, Slider, StyledInput } from '@erteam/ui'
import { open } from '@tauri-apps/plugin-dialog'
import { computed, ref, watch } from 'vue'

import ConfirmModalWrapper from '@/components/ui/modal/ConfirmModalWrapper.vue'
import { purge_cache_types } from '@/helpers/cache.js'
import { get, set } from '@/helpers/settings.ts'
import { showAppDbBackupsFolder } from '@/helpers/utils.js'
import i18n from '@/i18n.config'
import { useTheming } from '@/store/state'

const isRu = computed(() => (i18n.global.locale.value || '').startsWith('ru'))

const { handleError } = injectNotificationManager()
const themeStore = useTheming()
const settings = ref(await get())
const purgeCacheConfirmModal = ref(null)

watch(
	settings,
	async () => {
		const setSettings = JSON.parse(JSON.stringify(settings.value))

		if (!setSettings.custom_dir) {
			setSettings.custom_dir = null
		}

		await set(setSettings)
	},
	{ deep: true },
)

async function purgeCache() {
	await purge_cache_types([
		'project',
		'project_v3',
		'version',
		'user',
		'team',
		'organization',
		'file',
		'loader_manifest',
		'minecraft_manifest',
		'categories',
		'report_types',
		'loaders',
		'game_versions',
		'donation_platforms',
		'file_hash',
		'file_update',
		'search_results',
		'search_results_v3',
	]).catch(handleError)
}

function handlePurgeCacheClick() {
	if (themeStore.getFeatureFlag('skip_non_essential_warnings')) {
		void purgeCache()
		return
	}

	purgeCacheConfirmModal.value?.show()
}

async function openDbBackupsFolder() {
	await showAppDbBackupsFolder().catch(handleError)
}

async function findLauncherDir() {
	const newDir = await open({
		multiple: false,
		directory: true,
		title: 'Select a new app directory',
	})

	if (newDir) {
		settings.value.custom_dir = newDir
	}
}
</script>

<template>
	<div class="flex flex-col gap-6">
		<div class="flex flex-col gap-2.5">
			<h2 class="m-0 text-lg font-semibold text-contrast">
				{{ isRu ? 'Папка лаунчера' : 'App directory' }}
			</h2>
			<StyledInput
				id="appDir"
				v-model="settings.custom_dir"
				:icon="BoxIcon"
				type="text"
				wrapper-class="w-full"
			>
				<template #right>
					<ButtonStyled circular>
						<button class="ml-1.5" :title="isRu ? 'Выбрать другую папку' : 'Select a new app directory'" @click="findLauncherDir">
							<FolderSearchIcon />
						</button>
					</ButtonStyled>
				</template>
			</StyledInput>
			<p class="m-0 leading-tight text-sm text-secondary">
				{{ isRu ? 'Папка, где лаунчер сохраняет все свои файлы, сборки и кэш. Изменения вступят в силу после перезапуска.' : 'The directory where the launcher stores all of its files. Changes will be applied after restarting the launcher.' }}
			</p>
		</div>

		<div class="flex flex-col gap-2.5">
			<ConfirmModalWrapper
				ref="purgeCacheConfirmModal"
				:title="isRu ? 'Вы уверены, что хотите очистить кэш?' : 'Are you sure you want to purge the cache?'"
				:description="isRu ? 'Все временные файлы кэша будут удалены. Лаунчер заново загрузит актуальные данные.' : 'If you proceed, your entire cache will be purged. This may slow down the app temporarily.'"
				:has-to-type="false"
				:proceed-label="isRu ? 'Очистить кэш' : 'Purge cache'"
				:show-ad-on-close="false"
				@proceed="purgeCache"
			/>
			<h2 class="m-0 text-lg font-semibold text-contrast">
				{{ isRu ? 'Кэш приложения' : 'App cache' }}
			</h2>
			<button id="purge-cache" class="btn min-w-max" @click="handlePurgeCacheClick">
				<TrashIcon />
				{{ isRu ? 'Очистить кэш' : 'Purge cache' }}
			</button>
			<p class="m-0 leading-tight text-sm text-secondary">
				{{ isRu ? 'EndRage Launcher сохраняет локальный кэш данных для ускорения работы. Очистка принудительно обновит все каталоги и списки.' : 'The app stores a cache of data to speed up loading. This can be purged to force the app to reload data.' }}
			</p>
		</div>

		<div class="flex flex-col gap-2.5">
			<h2 class="m-0 text-lg font-semibold text-contrast mt-4">
				{{ isRu ? 'Максимум параллельных загрузок' : 'Maximum concurrent downloads' }}
			</h2>
			<Slider
				id="max-downloads"
				v-model="settings.max_concurrent_downloads"
				:min="1"
				:max="10"
				:step="1"
			/>
			<p class="m-0 leading-tight text-sm text-secondary">
				{{ isRu ? 'Количество файлов, одновременно загружаемых лаунчером. Уменьшите значение при медленном интернет-соединении.' : 'The maximum amount of files the launcher can download at the same time. Set this to a lower value if you have a poor internet connection.' }}
			</p>
		</div>

		<div class="flex flex-col gap-2.5">
			<h2 class="mt-0 m-0 text-lg font-semibold text-contrast">
				{{ isRu ? 'Максимум одновременных записей на диск' : 'Maximum concurrent writes' }}
			</h2>
			<Slider
				id="max-writes"
				v-model="settings.max_concurrent_writes"
				:min="1"
				:max="50"
				:step="1"
			/>
			<p class="m-0 leading-tight text-sm text-secondary">
				{{ isRu ? 'Количество файлов, одновременно записываемых на диск. Уменьшите значение, если возникают ошибки ввода-вывода (I/O).' : 'The maximum amount of files the launcher can write to the disk at once. Set this to a lower value if you are frequently getting I/O errors.' }}
			</p>
		</div>

		<div class="flex flex-col gap-2.5">
			<h2 class="mt-0 m-0 text-lg font-semibold text-contrast">
				{{ isRu ? 'Резервные копии базы данных' : 'App database backups' }}
			</h2>
			<button id="open-db-backups-folder" class="btn min-w-max" @click="openDbBackupsFolder">
				<FolderOpenIcon />
				{{ isRu ? 'Открыть папку с бэкапами' : 'Open backups folder' }}
			</button>
			<p class="m-0 leading-tight text-sm text-secondary">
				{{ isRu ? 'Здесь хранятся резервные копии важной информации лаунчера для быстрого восстановления при необходимости.' : 'Backups of important app data are stored here in case you need to recover them later.' }}
			</p>
		</div>
	</div>
</template>
