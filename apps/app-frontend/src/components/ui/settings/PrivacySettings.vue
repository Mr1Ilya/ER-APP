<script setup lang="ts">
import { Toggle } from '@erteam/ui'
import { computed, ref, watch } from 'vue'

import { optInAnalytics, optOutAnalytics } from '@/helpers/analytics'
import { get, set } from '@/helpers/settings.ts'
import i18n from '@/i18n.config'

const isRu = computed(() => (i18n.global.locale.value || '').startsWith('ru'))

const settings = ref(await get())

watch(
	settings,
	async () => {
		if (settings.value.telemetry) {
			optInAnalytics()
		} else {
			optOutAnalytics()
		}

		await set(settings.value)
	},
	{ deep: true },
)
</script>

<template>
	<div class="flex items-center justify-between gap-4">
		<div>
			<h2 class="m-0 text-lg font-semibold text-contrast">
				{{ isRu ? 'Персонализированные предложения' : 'Personalized ads' }}
			</h2>
			<p class="m-0 mt-1 text-sm text-secondary">
				{{ isRu ? 'Показ релевантных рекомендаций и предложений на основе ваших предпочтений.' : 'Shows ads and recommendations based on your preferences. By disabling this, ads will no longer be customized.' }}
			</p>
		</div>
		<Toggle id="personalized-ads" v-model="settings.personalized_ads" />
	</div>

	<div class="mt-6 flex items-center justify-between gap-4">
		<div>
			<h2 class="m-0 text-lg font-semibold text-contrast">
				{{ isRu ? 'Телеметрия и диагностика' : 'Telemetry' }}
			</h2>
			<p class="m-0 mt-1 text-sm text-secondary">
				{{ isRu ? 'Сбор анонимной аналитики и отчётов о сбоях для улучшения стабильности лаунчера.' : 'Collects anonymized analytics and usage data to improve user experience and app stability.' }}
			</p>
		</div>
		<Toggle id="opt-out-analytics" v-model="settings.telemetry" />
	</div>

	<div class="mt-6 flex items-center justify-between gap-4">
		<div>
			<h2 class="m-0 text-lg font-semibold text-contrast">
				{{ isRu ? 'Интеграция с Discord (RPC)' : 'Discord RPC' }}
			</h2>
			<p class="m-0 mt-1 text-sm text-secondary">
				{{ isRu ? 'Отображение текущей сборки и статуса EndRage Launcher в вашем профиле Discord.' : 'Manages the Discord Rich Presence integration in your Discord profile.' }}
			</p>
			<p class="m-0 mt-1 text-xs text-secondary opacity-75">
				{{ isRu ? 'Примечание: для применения может потребоваться перезапуск приложения.' : 'Note: app restart may be required to take effect.' }}
			</p>
		</div>
		<Toggle id="disable-discord-rpc" v-model="settings.discord_rpc" />
	</div>
</template>
