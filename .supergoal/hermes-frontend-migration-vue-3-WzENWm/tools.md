# Available Tools

## Detected tools (Stage 0)

- **codebase_search**: Available (semantic code search)
- **search_files**: Available (regex file search)
- **list_files**: Available
- **read_file**: Available
- **execute_command**: Available
- **write_to_file**: Available
- **apply_diff**: Available
- **ask_followup_question**: Available
- **new_task**: Available (subagent dispatch)
- **skill**: Available

## Not available
- **WebSearch**: NOT available
- **WebFetch**: NOT available  
- **Context7**: NOT available
- **MCP clients**: NOT detected

## Implications
- Web research is skipped — rely on training-cutoff knowledge of Vue 3, TanStack, Tailwind, shadcn-vue
- All planning evidence comes from local recon + codebase_search + file reading
- Subagent dispatch (new_task) available for parallel work within phases
