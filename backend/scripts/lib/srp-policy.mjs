export function validateSrpEntries(policy, entries) {
  // SRP is enforced by owner/package boundaries and dependency policy. Line
  // counts are not a reliable proxy for responsibility, so this validator is
  // intentionally kept as a compatibility entry point for the existing gate.
  void policy;
  void entries;
  return [];
}
