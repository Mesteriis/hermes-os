export type CommunicationTemplate = {
  template_id: string
  name: string
  subject_template: string
  body_template: string
  variables: string[]
  placeholder_variables: string[]
  undeclared_variables: string[]
  unused_variables: string[]
  malformed_placeholders: string[]
  language: string | null
  created_at: string
  updated_at: string
}

export type RichTemplateRenderRequest = {
  template_id: string
  variables: Record<string, string>
}

export type RichTemplateUpsertRequest = {
  template_id?: string
  name: string
  subject_template: string
  body_template: string
  variables: string[]
  language: string | null
}

export type RichTemplateUpsertResponse = {
  saved: boolean
  template: CommunicationTemplate
}

export type RichTemplateDeleteResponse = {
  template_id: string
  deleted: boolean
}

export type RichTemplateRenderResponse = {
  template_id: string
  variables: Record<string, string>
  rendered: {
    subject: string
    body: string
    missing_variables: string[]
    unresolved_variables: string[]
    malformed_placeholders: string[]
  }
}

export type RichTemplateMailMergePreviewRow = {
  row_id: string
  variables: Record<string, string>
}

export type RichTemplateMailMergePreviewRequest = {
  template_id: string
  rows: RichTemplateMailMergePreviewRow[]
}

export type RichTemplateMailMergePreviewItem = {
  row_id: string
  ready: boolean
  rendered: RichTemplateRenderResponse['rendered']
}

export type RichTemplateMailMergePreviewResponse = {
  template_id: string
  row_count: number
  ready_count: number
  blocked_count: number
  items: RichTemplateMailMergePreviewItem[]
}
