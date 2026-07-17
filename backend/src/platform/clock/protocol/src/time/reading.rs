//! Clock values and discontinuity policy.

pub const DEFAULT_JUMP_TOLERANCE_MILLIS: u64 = 1_000;
pub const DEFAULT_SUSPEND_GAP_MILLIS: u64 = 30_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct UtcMillisV1(i64);

impl UtcMillisV1 {
    #[must_use]
    pub const fn new(value: i64) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn value(self) -> i64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct MonotonicMillisV1(u64);

impl MonotonicMillisV1 {
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn value(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClockDiscontinuityV1 {
    Stable,
    WallClockJumpForward,
    WallClockJumpBackward,
    SuspendOrMonotonicGap,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ClockReadingV1 {
    wall_utc: UtcMillisV1,
    monotonic: MonotonicMillisV1,
    discontinuity: ClockDiscontinuityV1,
}

impl ClockReadingV1 {
    #[must_use]
    pub const fn new(
        wall_utc: UtcMillisV1,
        monotonic: MonotonicMillisV1,
        discontinuity: ClockDiscontinuityV1,
    ) -> Self {
        Self {
            wall_utc,
            monotonic,
            discontinuity,
        }
    }

    #[must_use]
    pub const fn wall_utc(self) -> UtcMillisV1 {
        self.wall_utc
    }

    #[must_use]
    pub const fn monotonic(self) -> MonotonicMillisV1 {
        self.monotonic
    }

    #[must_use]
    pub const fn discontinuity(self) -> ClockDiscontinuityV1 {
        self.discontinuity
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ClockPolicyV1 {
    jump_tolerance_millis: u64,
    suspend_gap_millis: u64,
}

impl ClockPolicyV1 {
    pub fn new(
        jump_tolerance_millis: u64,
        suspend_gap_millis: u64,
    ) -> Result<Self, ClockPolicyErrorV1> {
        if jump_tolerance_millis == 0 {
            return Err(ClockPolicyErrorV1::ZeroJumpTolerance);
        }
        if suspend_gap_millis < jump_tolerance_millis {
            return Err(ClockPolicyErrorV1::SuspendGapBelowJumpTolerance);
        }
        Ok(Self {
            jump_tolerance_millis,
            suspend_gap_millis,
        })
    }

    #[must_use]
    pub const fn production_default() -> Self {
        Self {
            jump_tolerance_millis: DEFAULT_JUMP_TOLERANCE_MILLIS,
            suspend_gap_millis: DEFAULT_SUSPEND_GAP_MILLIS,
        }
    }

    #[must_use]
    pub const fn jump_tolerance_millis(self) -> u64 {
        self.jump_tolerance_millis
    }

    #[must_use]
    pub const fn suspend_gap_millis(self) -> u64 {
        self.suspend_gap_millis
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClockPolicyErrorV1 {
    ZeroJumpTolerance,
    SuspendGapBelowJumpTolerance,
}
