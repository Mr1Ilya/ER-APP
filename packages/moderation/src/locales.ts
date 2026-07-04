import type { CrowdinMessages } from '@erteam/ui'

export const moderationLocaleModules = import.meta.glob<{ default: CrowdinMessages }>(
	'./locales/*/index.json',
	{ eager: false },
)
