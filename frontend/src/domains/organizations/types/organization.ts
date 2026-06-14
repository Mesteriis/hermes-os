export interface Organization {
  organization_id: string
  display_name: string
  industry?: string | null
  country?: string | null
  status?: string | null
  watchlist?: boolean | null
  health_status?: string | null
  description?: string | null
  website?: string | null
  legal_name?: string | null
  registration_number?: string | null
  vat?: string | null
  interaction_count?: number | null
  priority?: string | null
}
