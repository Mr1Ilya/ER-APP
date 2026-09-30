import { defineStore } from 'pinia'

import { findMinecraftAuthError } from '@/components/ui/minecraft-auth-error-modal/minecraft-auth-errors'
import { reportLauncherError } from '@/helpers/analytics'

export const useError = defineStore('errorsStore', {
	state: () => ({
		errorModal: null,
		minecraftAuthErrorModal: null,
	}),
	actions: {
		setErrorModal(ref) {
			this.errorModal = ref
		},
		setMinecraftAuthErrorModal(ref) {
			this.minecraftAuthErrorModal = ref
		},
		showError(error, context, closable = true, source = null) {
			reportLauncherError({
				error,
				context: typeof context === 'string' ? context : (source || (context && typeof context === 'object' && context.instanceId ? `Instance ${context.instanceId}` : 'Launcher Error')),
				extra: typeof context === 'object' ? context : undefined
			})

			if (
				error?.message &&
				(error.message.includes('Minecraft authentication error:') ||
					findMinecraftAuthError(error.message)) &&
				this.minecraftAuthErrorModal
			) {
				this.minecraftAuthErrorModal.show(error)
				return
			}
			if (this.errorModal) {
				this.errorModal.show(error, context, closable, source)
			}
		},
	},
})

export const handleSevereError = (err, context) => {
	const error = useError()
	error.showError(err, context)
	console.error(err)
}
