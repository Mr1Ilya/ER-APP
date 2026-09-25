<script setup>
import { injectNotificationManager } from '@erteam/ui'
import { computed, ref } from 'vue'

import JavaSelector from '@/components/ui/JavaSelector.vue'
import { get_java_versions, set_java_version } from '@/helpers/jre'
import i18n from '@/i18n.config'

const isRu = computed(() => (i18n.global.locale.value || '').startsWith('ru'))

const { handleError } = injectNotificationManager()

const javaVersions = ref(await get_java_versions().catch(handleError))
async function updateJavaVersion(version) {
	if (version?.path === '') {
		version.path = undefined
	}

	if (version?.path) {
		version.path = version.path.replace('java.exe', 'javaw.exe')
	}

	await set_java_version(version).catch(handleError)
}
</script>
<template>
	<div class="flex flex-col gap-6">
		<div
			v-for="(javaVersion, index) in [25, 21, 17, 8]"
			:key="`java-${javaVersion}`"
			class="flex flex-col gap-2.5"
		>
			<h2 class="m-0 text-lg font-semibold text-contrast" :class="{ 'mt-4': index !== 0 }">
				{{ isRu ? `Расположение Java ${javaVersion}` : `Java ${javaVersion} location` }}
			</h2>
			<JavaSelector
				:id="'java-selector-' + javaVersion"
				v-model="javaVersions[javaVersion]"
				:version="javaVersion"
				@update:model-value="updateJavaVersion"
			/>
		</div>
	</div>
</template>
