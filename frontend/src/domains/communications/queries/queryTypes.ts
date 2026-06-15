import type { MaybeRefOrGetter } from 'vue'

export type QueryParam<T> = MaybeRefOrGetter<T | undefined>
export type NullableQueryParam<T> = MaybeRefOrGetter<T | null>
