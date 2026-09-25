<script setup lang="ts">
import { PlusIcon } from '@erteam/assets'
import { ButtonStyled, injectNotificationManager, NavTabs } from '@erteam/ui'
import { computed, inject, onUnmounted, ref, shallowRef, watch } from 'vue'
import { useRoute } from 'vue-router'

import { NewInstanceImage } from '@/assets/icons'
import { instance_listener } from '@/helpers/events.js'
import { list } from '@/helpers/instance'
import i18n from '@/i18n.config'
import { useBreadcrumbs } from '@/store/breadcrumbs.js'

const { handleError } = injectNotificationManager()
const showCreationModal = inject('showCreationModal')
const route = useRoute()
const breadcrumbs = useBreadcrumbs()

const isRu = computed(() => (i18n.global.locale.value || '').startsWith('ru'))

watch(
	isRu,
	(ru) => {
		breadcrumbs.setRootContext({ name: ru ? 'Мои сборки' : 'Library', link: route.path })
	},
	{ immediate: true },
)

const instances = shallowRef(await list().catch(handleError))

const offline = ref(!navigator.onLine)
window.addEventListener('offline', () => {
	offline.value = true
})
window.addEventListener('online', () => {
	offline.value = false
})

const unlistenInstance = await instance_listener(async () => {
	instances.value = await list().catch(handleError)
})
onUnmounted(() => {
	unlistenInstance()
})
</script>

<template>
	<div class="p-6 flex flex-col gap-3">
		<h1 class="m-0 text-2xl hidden">{{ isRu ? 'Мои сборки' : 'Library' }}</h1>
		<NavTabs
			:links="[
				{ label: isRu ? 'Все сборки' : 'All', href: `/library` },
				{ label: isRu ? 'Модпаки' : 'Modpacks', href: `/library/modpacks` },
				{ label: isRu ? 'Серверы' : 'Servers', href: `/library/servers` },
				{ label: isRu ? 'Свои' : 'Custom', href: `/library/custom` },
				{ label: 'Shared with me', href: `/library/shared`, shown: false },
				{ label: 'Saved', href: `/library/saved`, shown: false },
			]"
		/>
		<template v-if="instances && instances.length > 0">
			<RouterView v-if="route.path.startsWith('/library')" :instances="instances" />
		</template>
		<div v-else class="no-instance">
			<div class="icon">
				<NewInstanceImage />
			</div>
			<h3>{{ isRu ? 'Нет установленных сборок' : 'No installed instances' }}</h3>
			<ButtonStyled color="brand">
				<button :disabled="offline" @click="showCreationModal?.()">
					<PlusIcon />
					{{ isRu ? 'Создать новую сборку' : 'Create new instance' }}
				</button>
			</ButtonStyled>
		</div>
	</div>
</template>

<style lang="scss" scoped>
.no-instance {
	display: flex;
	flex-direction: column;
	align-items: center;
	justify-content: center;
	height: 100%;
	gap: var(--gap-md);

	p,
	h3 {
		margin: 0;
	}

	.icon {
		svg {
			width: 10rem;
			height: 10rem;
		}
	}
}
</style>
