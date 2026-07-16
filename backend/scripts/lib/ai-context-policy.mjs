import { list, violation } from './validation-diagnostics.mjs';

const AI_CONTEXT_KEYS = [
  'owner',
  'acquisitionMode',
  'assemblyOwnerRole',
  'workflowAssembly',
  'contractScope',
  'wireShape',
  'commonReceiptContract',
  'contextPayload',
  'schemaBinding',
  'globalFragmentUnionEnabled',
  'opaquePayloadBytesEnabled',
  'assembledContextLifetime',
  'consistencyModel',
  'largePrivateContentTransport',
  'businessTruthPromotion',
  'aiOutputAuthority',
  'aiDirectOwnerQueryAccessEnabled',
  'aiDirectCrossOwnerQueryOrchestrationEnabled',
  'aiCrossOwnerSqlEnabled',
  'genericContextApiEnabled',
  'durableContextProjectionEnabled',
  'remoteEgressRequiresExplicitPolicy',
];

function hasExactKeys(value, expectedKeys) {
  if (!value || typeof value !== 'object' || Array.isArray(value)) return false;
  const keys = Object.keys(value);
  return keys.length === expectedKeys.length
    && keys.every((key) => expectedKeys.includes(key));
}

export function validateAiContextPolicy(policy) {
  const aiContext = policy?.aiContext;
  const valid = hasExactKeys(aiContext, AI_CONTEXT_KEYS)
    && aiContext.owner === 'ai'
    && list(policy?.domains?.registered).includes(aiContext.owner)
    && list(policy?.domains?.developmentAllowlist).includes(aiContext.owner)
    && list(policy?.projections?.blockedOwners).includes('context')
    && aiContext.acquisitionMode === 'workflow_supplied_typed_ai_request_v1'
    && aiContext.assemblyOwnerRole === 'workflow'
    && aiContext.workflowAssembly === 'explicit_owner_query_ports_per_use_case'
    && aiContext.contractScope === 'typed_single_use_case'
    && aiContext.wireShape === 'common_receipt_plus_distinct_generated_use_case_request'
    && aiContext.commonReceiptContract === 'AiContextReceiptV1'
    && aiContext.contextPayload === 'concrete_use_case_message_field'
    && aiContext.schemaBinding === 'exact_request_message_type_revision_and_schema_sha256'
    && aiContext.globalFragmentUnionEnabled === false
    && aiContext.opaquePayloadBytesEnabled === false
    && aiContext.assembledContextLifetime === 'request_or_run_scoped_not_projection'
    && aiContext.consistencyModel === 'as_of_source_revisions_and_completeness'
    && aiContext.largePrivateContentTransport === 'expiring_blob_ref_after_blob_v1'
    && aiContext.businessTruthPromotion === 'workflow_to_target_domain_command_or_review'
    && aiContext.aiOutputAuthority === 'candidate_only'
    && aiContext.aiDirectOwnerQueryAccessEnabled === false
    && aiContext.aiDirectCrossOwnerQueryOrchestrationEnabled === false
    && aiContext.aiCrossOwnerSqlEnabled === false
    && aiContext.genericContextApiEnabled === false
    && aiContext.durableContextProjectionEnabled === false
    && aiContext.remoteEgressRequiresExplicitPolicy === true;

  return valid ? [] : [violation(
    'ai_context_policy',
    'aiContext',
    'AI context must be bounded, use-case-specific and assembled by workflows through explicit owner query contracts',
  )];
}
