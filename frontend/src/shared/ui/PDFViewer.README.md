# PDFViewer

## Description
Generic PDF preview shell using the browser PDF object renderer.

## When to use
Use for safe local or already-authorized PDF URLs.

## When not to use
Do not fetch provider bytes or perform document indexing here.

## Accessibility
Provide a title that describes the document.

## Keyboard
Browser PDF controls own keyboard interaction.

## Examples
`<PDFViewer :src="pdfUrl" title="Contract" />`

## Anti Patterns
Do not store document truth or OCR state here.
