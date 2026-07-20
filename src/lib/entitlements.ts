import type { CloudUser, Entitlements } from "@/types";

const _DEFAULT_REQUESTS_PER_HOUR = 100;

interface Capabilities {
  browserAutomation: boolean;
  crossOsFingerprints: boolean;
  cloudBackup: boolean;
  teamCollaboration: boolean;
}

const _NONE: Entitlements = {
  active: false,
  browserAutomation: false,
  crossOsFingerprints: false,
  cloudBackup: false,
  teamCollaboration: false,
  profileLimit: 0,
  requestsPerHour: 0,
};

// Mirror of PLAN_CAPABILITIES in apps/backend/src/plans/entitlements.ts. Keep in
// sync — a new plan must be declared here too, or it falls back to DEFAULT_PAID.
const _PLAN_CAPABILITIES: Record<string, Capabilities> = {
  starter: {
    browserAutomation: false,
    crossOsFingerprints: true,
    cloudBackup: true,
    teamCollaboration: false,
  },
  pro: {
    browserAutomation: true,
    crossOsFingerprints: true,
    cloudBackup: true,
    teamCollaboration: false,
  },
  team: {
    browserAutomation: true,
    crossOsFingerprints: true,
    cloudBackup: true,
    teamCollaboration: true,
  },
  enterprise: {
    browserAutomation: true,
    crossOsFingerprints: true,
    cloudBackup: true,
    teamCollaboration: true,
  },
};

// Unknown paid plan -> pro-level (never team), matching the backend default.
const _DEFAULT_PAID: Capabilities = {
  browserAutomation: true,
  crossOsFingerprints: true,
  cloudBackup: true,
  teamCollaboration: false,
};

/**
 * The user's effective entitlements. Prefers the backend-resolved object the
 * desktop attaches to CloudUser; only falls back to deriving from the plan
 * fields when it's missing (older cached state). The fallback mirrors the
 * backend matrix in `apps/backend/src/plans/entitlements.ts`.
 *
 * ponytail: local dev/test override — always returns fully-unlocked Pro
 * entitlements so every UI gate (automation, cross-OS, cloud backup, team)
 * passes without a paid cloud account. Revert to the original body (below,
 * commented) before shipping.
 */
export function getEntitlements(
  _user: CloudUser | null | undefined,
): Entitlements {
  return {
    active: true,
    browserAutomation: true,
    crossOsFingerprints: true,
    cloudBackup: true,
    teamCollaboration: true,
    profileLimit: 1_000_000,
    requestsPerHour: 100,
  };
}

/*
// --- original implementation (restore before shipping) ---
export function _getEntitlementsOriginal(
  user: CloudUser | null | undefined,
): Entitlements {
  if (user?.entitlements) return user.entitlements;
  if (!user) return NONE;

  const active =
    user.plan !== "free" &&
    (user.subscriptionStatus === "active" || user.planPeriod === "lifetime");
  if (!active) return NONE;

  const caps = PLAN_CAPABILITIES[user.plan] ?? DEFAULT_PAID;
  return {
    active: true,
    browserAutomation: caps.browserAutomation,
    crossOsFingerprints: caps.crossOsFingerprints,
    cloudBackup: caps.cloudBackup,
    teamCollaboration: caps.teamCollaboration,
    profileLimit: user.profileLimit,
    requestsPerHour: caps.browserAutomation ? DEFAULT_REQUESTS_PER_HOUR : 0,
  };
}
*/
