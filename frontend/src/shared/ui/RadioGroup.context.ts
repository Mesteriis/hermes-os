import type { ComputedRef, InjectionKey } from 'vue'

export interface HermesRadioGroupContext {
	name: string
	modelValue: ComputedRef<string | number | null>
	disabled: ComputedRef<boolean>
	select(value: string | number): void
}

export const hermesRadioGroupKey: InjectionKey<HermesRadioGroupContext> = Symbol('hermes-radio-group')
