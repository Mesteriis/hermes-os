import { onBeforeUnmount, watch, type Ref } from 'vue'

const DEFAULT_DISMISS_DELAY_MS = 140
const DEFAULT_BOUNDARY_PADDING_PX = 50

type BoundaryElement = HTMLElement | { $el?: Element | null } | null | undefined

type MouseLeaveDismissOptions = {
	isOpen?: Readonly<Ref<boolean>>
	getBoundaryElements?: () => BoundaryElement[]
	boundaryPaddingPx?: number
}

export function useMouseLeaveDismiss(
	close: () => void,
	delayMs = DEFAULT_DISMISS_DELAY_MS,
	options?: MouseLeaveDismissOptions
) {
	let dismissTimer: number | undefined
	let stopMouseMoveTracking: (() => void) | undefined
	const boundaryPaddingPx = options?.boundaryPaddingPx ?? DEFAULT_BOUNDARY_PADDING_PX

	function cancelMouseLeaveDismiss(): void {
		if (dismissTimer === undefined) {
			return
		}

		window.clearTimeout(dismissTimer)
		dismissTimer = undefined
	}

	function scheduleMouseLeaveDismiss(event?: MouseEvent): void {
		if (event && isInsideBoundary(event.relatedTarget, event.clientX, event.clientY)) {
			cancelMouseLeaveDismiss()
			return
		}

		if (dismissTimer !== undefined) {
			return
		}

		dismissTimer = window.setTimeout(() => {
			dismissTimer = undefined
			close()
		}, delayMs)
	}

	function closeAfterMouseLeave(): void {
		cancelMouseLeaveDismiss()
		close()
	}

	function getBoundaryHtmlElements(): HTMLElement[] {
		return options?.getBoundaryElements?.()
			.map((element) => {
				if (element instanceof HTMLElement) {
					return element
				}

				if (element?.$el instanceof HTMLElement) {
					return element.$el
				}

				return undefined
			})
			.filter((element): element is HTMLElement => Boolean(element)) ?? []
	}

	function isPointInsideExpandedBoundary(clientX: number, clientY: number): boolean {
		return getBoundaryHtmlElements().some((element) => {
			const rect = element.getBoundingClientRect()

			return (
				clientX >= rect.left - boundaryPaddingPx &&
				clientX <= rect.right + boundaryPaddingPx &&
				clientY >= rect.top - boundaryPaddingPx &&
				clientY <= rect.bottom + boundaryPaddingPx
			)
		})
	}

	function isInsideBoundary(
		target: EventTarget | null,
		clientX?: number,
		clientY?: number
	): boolean {
		if (typeof clientX === 'number' && typeof clientY === 'number' && isPointInsideExpandedBoundary(clientX, clientY)) {
			return true
		}

		if (!(target instanceof Node)) {
			return false
		}

		return getBoundaryHtmlElements().some((element) => element.contains(target))
	}

	function handleWindowMouseMove(event: MouseEvent): void {
		if (!options?.isOpen?.value) {
			return
		}

		if (isInsideBoundary(event.target, event.clientX, event.clientY)) {
			cancelMouseLeaveDismiss()
			return
		}

		scheduleMouseLeaveDismiss(event)
	}

	function startMouseMoveTracking(): void {
		if (stopMouseMoveTracking || !options?.getBoundaryElements) {
			return
		}

		window.addEventListener('mousemove', handleWindowMouseMove, { passive: true })
		stopMouseMoveTracking = () => {
			window.removeEventListener('mousemove', handleWindowMouseMove)
			stopMouseMoveTracking = undefined
		}
	}

	function stopTrackingMouseMove(): void {
		stopMouseMoveTracking?.()
		cancelMouseLeaveDismiss()
	}

	if (options?.isOpen && options.getBoundaryElements) {
		watch(
			options.isOpen,
			(isOpen) => {
				if (isOpen) {
					startMouseMoveTracking()
					return
				}

				stopTrackingMouseMove()
			},
			{ immediate: true }
		)
	}

	onBeforeUnmount(stopTrackingMouseMove)

	return {
		cancelMouseLeaveDismiss,
		scheduleMouseLeaveDismiss,
		closeAfterMouseLeave
	}
}
