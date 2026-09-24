<script setup lang="ts">
import { ref } from 'vue'
import AccountsCard from '@/components/ui/AccountsCard.vue'

const isVisible = ref(false)

function show() {
	isVisible.value = true
}

function hide() {
	isVisible.value = false
}

defineExpose({
	show,
	hide
})
</script>

<template>
	<Teleport to="body">
		<Transition name="fade">
			<div
				v-if="isVisible"
				class="fixed inset-0 z-[150] flex items-center justify-center bg-black/70 backdrop-blur-sm p-4"
				@click.self="hide"
			>
				<div class="w-full max-w-md bg-[#1a1c21] border border-[#2a2d36] rounded-3xl p-6 shadow-2xl flex flex-col gap-4 text-white">
					<!-- Header -->
					<div class="flex items-center justify-between pb-3 border-b border-[#282a32]">
						<div class="flex items-center gap-2.5">
							<div class="w-8 h-8 rounded-xl bg-[#22c55e]/15 text-[#22c55e] flex items-center justify-center">
								<svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
									<path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"></path>
									<circle cx="12" cy="7" r="4"></circle>
								</svg>
							</div>
							<h3 class="text-base font-bold text-white m-0">Аккаунты Minecraft</h3>
						</div>
						<button
							class="w-8 h-8 rounded-xl bg-transparent hover:bg-white/10 text-gray-400 hover:text-white flex items-center justify-center transition-colors border-0 cursor-pointer"
							@click="hide"
						>
							<svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
								<line x1="18" y1="6" x2="6" y2="18"></line>
								<line x1="6" y1="6" x2="18" y2="18"></line>
							</svg>
						</button>
					</div>

					<!-- Accounts Card -->
					<div class="accounts-modal-content">
						<Suspense>
							<AccountsCard />
						</Suspense>
					</div>
				</div>
			</div>
		</Transition>
	</Teleport>
</template>

<style scoped>
.fade-enter-active,
.fade-leave-active {
	transition: opacity 0.2s ease, transform 0.2s ease;
}
.fade-enter-from,
.fade-leave-to {
	opacity: 0;
	transform: scale(0.95);
}
</style>
