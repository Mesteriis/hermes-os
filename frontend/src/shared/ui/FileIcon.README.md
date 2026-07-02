# FileIcon

## Description
File type icon primitive with MIME-aware fallback.

## When to use
Use beside file names, attachments and local artifacts.

## When not to use
Do not inspect file bytes or open files from this component.

## Accessibility
The wrapper exposes an accessible image label.

## Keyboard
No keyboard behavior.

## Examples
`<FileIcon mime-type="application/pdf" label="PDF" />`

## Anti Patterns
Do not treat MIME labels from untrusted input as validated truth.
