# NoSearchResultsState

## Description
Search-specific empty result state.

## When to use
Use when a local query returns no visible matches.

## When not to use
Do not use for initial empty datasets.

## Accessibility
Uses polite status semantics.

## Keyboard
Search input keyboard behavior belongs to the owner.

## Examples
`<NoSearchResultsState query="project atlas" />`

## Anti Patterns
Do not mutate filters from inside this display state.
