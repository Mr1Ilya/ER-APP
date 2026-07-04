<script setup>
import { onMounted, ref } from 'vue'

import { init_ads_window } from '@/helpers/ads.js'

const adsWrapper = ref(null)

let devicePixelRatioWatcher = null

function initDevicePixelRatioWatcher() {
	if (devicePixelRatioWatcher) {
		devicePixelRatioWatcher.removeEventListener('change', updateAdPosition)
	}

	devicePixelRatioWatcher = window.matchMedia(`(resolution: ${window.devicePixelRatio}dppx)`)
	devicePixelRatioWatcher.addEventListener('change', updateAdPosition)
}

onMounted(() => {
	updateAdPosition()

	window.addEventListener('resize', updateAdPosition)
	initDevicePixelRatioWatcher()
})

function updateAdPosition() {
	if (adsWrapper.value) {
		init_ads_window()
		initDevicePixelRatioWatcher()
	}
}
</script>

<template>
	<div class="hidden"></div>
</template>
<style lang="scss" scoped>
.light,
.light-mode {
	.dark-image {
		display: none;
	}

	.light-image {
		display: block;
	}
}
</style>
