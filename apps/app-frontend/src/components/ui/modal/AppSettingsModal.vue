<script setup lang="ts">
import {
	CoffeeIcon,
	GameIcon,
	GaugeIcon,
	LanguagesIcon,
	ModrinthIcon,
	PaintbrushIcon,
	RefreshCwIcon,
	SettingsIcon,
	ShieldIcon,
	ToggleRightIcon,
} from '@erteam/assets'
import {
	commonMessages,
	commonSettingsMessages,
	defineMessage,
	defineMessages,
	ProgressBar,
	TabbedModal,
	useVIntl,
} from '@erteam/ui'
import { getVersion } from '@tauri-apps/api/app'
import { platform as getOsPlatform, version as getOsVersion } from '@tauri-apps/plugin-os'
import { computed, ref, watch } from 'vue'

import AppearanceSettings from '@/components/ui/settings/AppearanceSettings.vue'
import DefaultInstanceSettings from '@/components/ui/settings/DefaultInstanceSettings.vue'
import FeatureFlagSettings from '@/components/ui/settings/FeatureFlagSettings.vue'
import JavaSettings from '@/components/ui/settings/JavaSettings.vue'
import LanguageSettings from '@/components/ui/settings/LanguageSettings.vue'
import PrivacySettings from '@/components/ui/settings/PrivacySettings.vue'
import ResourceManagementSettings from '@/components/ui/settings/ResourceManagementSettings.vue'
import { get, set } from '@/helpers/settings.ts'
import { injectAppUpdateDownloadProgress } from '@/providers/download-progress.ts'
import { useTheming } from '@/store/state'

const themeStore = useTheming()

const { formatMessage } = useVIntl()

const devModeCounter = ref(0)

const developerModeEnabled = defineMessage({
	id: 'app.settings.developer-mode-enabled',
	defaultMessage: 'Developer mode enabled.',
})

import i18n from '@/i18n.config'

const isRu = computed(() => (i18n.global.locale.value || '').startsWith('ru'))

const tabs = computed(() => [
	{
		name: isRu.value ? 'Внешний вид' : 'Appearance',
		icon: PaintbrushIcon,
		content: AppearanceSettings,
	},
	{
		name: isRu.value ? 'Язык' : 'Language',
		icon: LanguagesIcon,
		content: LanguageSettings,
		badge: isRu.value ? 'Бета' : 'Beta',
	},
	{
		name: isRu.value ? 'Приватность' : 'Privacy',
		icon: ShieldIcon,
		content: PrivacySettings,
	},
	{
		name: isRu.value ? 'Установки Java' : 'Java installations',
		icon: CoffeeIcon,
		content: JavaSettings,
	},
	{
		name: isRu.value ? 'Параметры по умолчанию' : 'Default instance options',
		icon: GameIcon,
		content: DefaultInstanceSettings,
	},
	{
		name: isRu.value ? 'Управление ресурсами' : 'Resource management',
		icon: GaugeIcon,
		content: ResourceManagementSettings,
	},
	{
		name: isRu.value ? 'Флаги функций' : 'Feature flags',
		icon: ToggleRightIcon,
		content: FeatureFlagSettings,
		developerOnly: true,
	},
])

const modal = ref<InstanceType<typeof TabbedModal> | null>(null)

function show() {
	modal.value?.show()
}

defineExpose({ show })

const { progress, version: downloadingVersion } = injectAppUpdateDownloadProgress()

const version = await getVersion()
const osPlatform = getOsPlatform()
const osVersion = getOsVersion()
const settings = ref(await get())

watch(
	settings,
	async () => {
		await set(settings.value)
	},
	{ deep: true },
)

function devModeCount() {
	devModeCounter.value++
	if (devModeCounter.value > 5) {
		themeStore.devMode = !themeStore.devMode
		settings.value.developer_mode = !!themeStore.devMode
		devModeCounter.value = 0

		if (!themeStore.devMode && tabs.value[modal.value!.selectedTab].developerOnly) {
			modal.value!.setTab(0)
		}
	}
}

import {
	appUpdateState,
	downloadAvailableAppUpdate,
	installAvailableAppUpdate,
	triggerCheckForUpdates,
} from '@/providers/app-update'

const {
	isCheckingUpdates,
	availableUpdate,
	finishedDownloading,
	downloading,
	downloadPercent,
} = appUpdateState

const checkStatusMessage = ref<string | null>(null)

async function handleManualCheck() {
	if (finishedDownloading.value) {
		await installAvailableAppUpdate()
		return
	}
	if (availableUpdate.value && !downloading.value) {
		await downloadAvailableAppUpdate()
		return
	}

	checkStatusMessage.value = null
	const res = await triggerCheckForUpdates(true)
	if (res && res.updateFound) {
		checkStatusMessage.value = isRu.value ? `Найдено обновление v${res.version}!` : `Found update v${res.version}!`
	} else if (!availableUpdate.value) {
		const latestMsg = isRu.value ? 'У вас последняя версия' : 'You have the latest version'
		checkStatusMessage.value = latestMsg
		setTimeout(() => {
			if (checkStatusMessage.value === latestMsg) {
				checkStatusMessage.value = null
			}
		}, 4000)
	}
}

const updateButtonLabel = computed(() => {
	if (isCheckingUpdates.value) {
		return isRu.value ? 'Проверка...' : 'Checking...'
	}
	if (downloading.value) {
		return isRu.value ? `Загрузка ${downloadPercent.value}%` : `Downloading ${downloadPercent.value}%`
	}
	if (finishedDownloading.value) {
		return isRu.value ? 'Перезапустить' : 'Restart'
	}
	if (availableUpdate.value) {
		return isRu.value ? `Обновить до v${availableUpdate.value.version}` : `Update to v${availableUpdate.value.version}`
	}
	if (checkStatusMessage.value) {
		return checkStatusMessage.value
	}
	return isRu.value ? 'Проверить обновления' : 'Check for updates'
})

const messages = defineMessages({
	downloading: {
		id: 'app.settings.downloading',
		defaultMessage: 'Downloading v{version}',
	},
	title: {
		id: 'app.settings.title',
		defaultMessage: 'Settings',
	},
})
</script>
<template>
	<TabbedModal ref="modal" :tabs="tabs.filter((t) => !t.developerOnly || themeStore.devMode)">
		<template #title>
			<span class="flex items-center gap-2 text-lg font-extrabold text-contrast">
				<SettingsIcon class="w-5 h-5 text-contrast" /> {{ isRu ? 'Настройки' : 'Settings' }}
			</span>
		</template>
		<template #footer>
			<div class="mt-auto text-secondary text-sm">
				<div class="mb-3">
					<template v-if="progress > 0 && progress < 1">
						<p class="m-0 mb-2">
							{{ formatMessage(messages.downloading, { version: downloadingVersion }) }}
						</p>
						<ProgressBar :progress="progress" />
					</template>
				</div>
				<p v-if="themeStore.devMode" class="text-brand font-semibold m-0 mb-2">
					{{ formatMessage(developerModeEnabled) }}
				</p>
				<div class="flex items-center justify-between gap-3 pt-2">
					<div class="flex items-center gap-3">
						<button
							class="p-0 m-0 bg-transparent border-none cursor-pointer button-animation"
							:class="{
								'text-brand': themeStore.devMode,
								'text-secondary': !themeStore.devMode,
							}"
							@click="devModeCount"
						>
							<ModrinthIcon class="w-6 h-6" />
						</button>
						<div class="max-w-[200px]">
							<p class="m-0 font-medium text-contrast">EndRage Launcher {{ version }}</p>
							<p class="m-0 text-xs text-secondary">
								<span v-if="osPlatform === 'macos'">macOS</span>
								<span v-else class="capitalize">{{ osPlatform }}</span>
								{{ osVersion }}
							</p>
						</div>
					</div>
					<button
						type="button"
						class="px-3 py-1.5 rounded-xl text-xs font-medium transition-all cursor-pointer flex items-center gap-1.5 border"
						:class="[
							availableUpdate
								? 'bg-brand text-black border-brand font-semibold hover:brightness-110'
								: 'bg-surface-3 text-secondary hover:text-contrast hover:bg-surface-4 border-divider',
							isCheckingUpdates || downloading ? 'opacity-80 cursor-wait' : ''
						]"
						:disabled="isCheckingUpdates"
						@click="handleManualCheck"
					>
						<RefreshCwIcon class="w-3.5 h-3.5" :class="{ 'animate-spin': isCheckingUpdates || downloading }" />
						<span>{{ updateButtonLabel }}</span>
					</button>
				</div>
			</div>
		</template>
	</TabbedModal>
</template>
