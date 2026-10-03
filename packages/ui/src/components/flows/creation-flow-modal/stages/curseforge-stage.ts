import { LeftArrowIcon } from '@erteam/assets'
import { markRaw } from 'vue'

import { commonMessages } from '#ui/utils/common-messages'

import type { StageConfigInput } from '../../../base'
import CurseforgeModpackStage from '../components/CurseforgeModpackStage.vue'
import { type CreationFlowContextValue, creationFlowMessages } from '../creation-flow-context'

export const stageConfig: StageConfigInput<CreationFlowContextValue> = {
	id: 'curseforge-modpack',
	title: (ctx) => ctx.formatMessage(creationFlowMessages.chooseCurseforgeModpackTitle),
	stageContent: markRaw(CurseforgeModpackStage),
	skip: (ctx) => ctx.setupType.value !== 'curseforge' || ctx.isImportMode.value,
	leftButtonConfig: (ctx) => ({
		label: ctx.formatMessage(commonMessages.backButton),
		icon: LeftArrowIcon,
		onClick: () => ctx.modal.value?.setStage('setup-type'),
	}),
	rightButtonConfig: null,
	maxWidth: '560px',
}
