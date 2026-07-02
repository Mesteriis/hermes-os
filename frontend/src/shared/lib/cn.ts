type ClassDictionary = Record<string, boolean | null | undefined>
type ClassArray = ClassValue[]
export type ClassValue = string | number | false | null | undefined | ClassDictionary | ClassArray

export function cn(...inputs: ClassValue[]): string {
	const classes: string[] = []
	for (const input of inputs) {
		appendClassValue(input, classes)
	}
	return Array.from(new Set(classes)).join(' ')
}

function appendClassValue(input: ClassValue, classes: string[]): void {
	if (!input) return
	if (typeof input === 'string' || typeof input === 'number') {
		classes.push(String(input))
		return
	}
	if (Array.isArray(input)) {
		for (const item of input) appendClassValue(item, classes)
		return
	}
	for (const [className, enabled] of Object.entries(input)) {
		if (enabled) classes.push(className)
	}
}
