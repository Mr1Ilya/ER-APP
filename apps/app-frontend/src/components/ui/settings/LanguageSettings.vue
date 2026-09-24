<script setup lang="ts">
import { LanguageSelector, LOCALES } from '@erteam/ui'
import { computed, ref, watch } from 'vue'

import { get, set } from '@/helpers/settings.ts'
import i18n from '@/i18n.config'

const settings = ref(await get())

watch(
	settings,
	async () => {
		await set(settings.value)
	},
	{ deep: true },
)

const $isChanging = ref(false)

const allowedLocales = computed(() =>
	LOCALES.filter((l) => l.code === 'ru-RU' || l.code === 'en-US')
)

async function onLocaleChange(newLocale: string) {
	if (settings.value.locale === newLocale) return

	$isChanging.value = true
	try {
		localStorage.setItem('endrage_user_language_set', 'true')
		i18n.global.locale.value = newLocale
		settings.value.locale = newLocale
	} finally {
		$isChanging.value = false
	}
}
</script>

<template>
	<h2 class="m-0 text-lg font-bold text-white">Язык / Language</h2>
	<p class="m-0 mt-1 mb-4 text-sm text-[#9da3af]">
		Выберите предпочитаемый язык интерфейса для EndRage Launcher (доступны Русский и English).
	</p>

	<LanguageSelector
		:current-locale="settings.locale || 'ru-RU'"
		:locales="allowedLocales"
		:on-locale-change="onLocaleChange"
		:is-changing="$isChanging"
	/>
</template>
