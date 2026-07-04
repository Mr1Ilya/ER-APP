import { attributionQuickReplies } from '@erteam/moderation'
import { provideAttributionModeration } from '@erteam/ui'

export function setupAttributionModerationProvider() {
	provideAttributionModeration({ attributionQuickReplies })
}
