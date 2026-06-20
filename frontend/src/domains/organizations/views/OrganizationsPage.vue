<script setup lang="ts">
import { useI18n } from '../../../platform/i18n'
import Icon from '../../../shared/ui/Icon.vue'
import { useOrganizationsQuery } from '../queries/useOrganizationsQuery'
import OrganizationsList from '../components/OrganizationsList.vue'
import OrganizationsDetail from '../components/OrganizationsDetail.vue'
import type { Organization } from '../types/organization'
import { ref, computed } from 'vue'

const { t } = useI18n()

const { data: organizationsData, isLoading } = useOrganizationsQuery()

const selectedOrganizationId = ref('')

const selectedOrganization = computed(() => {
  const orgs = organizationsData.value ?? []
  return orgs.find((o: Organization) => o.organization_id === selectedOrganizationId.value) ?? orgs[0] ?? null
})

const orgPeople = computed(() => [])
</script>

<template>
  <section class="organizations-page">
    <div class="view-header">
      <div class="view-title-with-icon">
        <span class="hero-mark small"><Icon icon="tabler:building" :size="28" /></span>
        <div>
          <h1>{{ t('Companies') }}</h1>
          <p>{{ t('All companies and organizations from your communications') }}</p>
        </div>
      </div>
    </div>
    <div class="org-layout">
      <OrganizationsList
        :organizations="organizationsData ?? []"
        :selectedOrganizationId="selectedOrganizationId"
        :isOrganizationsLoading="isLoading"
        @selectOrg="(id) => { selectedOrganizationId = id }"
      />
      <OrganizationsDetail
        :selectedOrganization="selectedOrganization as unknown as Record<string, unknown>"
        :orgPeople="orgPeople as unknown as unknown[]"
      />
    </div>
  </section>
</template>

<style scoped>
.organizations-page {
  display: grid;
  grid-template-columns: repeat(var(--hh-layout-columns), minmax(0, 1fr));
  grid-auto-flow: row;
  grid-auto-rows: min-content;
  align-content: start;
  gap: var(--hh-layout-gap);
  height: 100%;
  min-height: 0;
  overflow: hidden;
  padding-right: 0;
}
.organizations-page > * {
  grid-column: 1 / -1;
  min-width: 0;
}
.org-layout {
  display: grid;
  grid-template-columns: 320px 1fr;
  gap: var(--hh-layout-gap);
  min-height: 0;
}
</style>
