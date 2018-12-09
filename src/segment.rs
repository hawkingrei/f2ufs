pub(crate) enum SegmentState {
    /// the segment is marked for reuse, should never receive
    /// new pids,
    /// TODO consider: but may receive removals for pids that were
    /// already removed?
    Free,

    /// the segment is being written to or actively recovered, and
    /// will have pages assigned to it
    Active,

    /// the segment is no longer being written to or recovered, and
    /// will have pages marked as relocated from it
    Inactive,

    /// the segment is having its resident pages relocated before
    /// becoming free
    Draining,
}

/* Notice: The order of dirty type is same with CURSEG_XXX in f2fs.h */
pub enum dirty_type {
    DIRTY_HOT_DATA,  /* dirty segments assigned as hot data logs */
    DIRTY_WARM_DATA, /* dirty segments assigned as warm data logs */
    DIRTY_COLD_DATA, /* dirty segments assigned as cold data logs */
    DIRTY_HOT_NODE,  /* dirty segments assigned as hot node logs */
    DIRTY_WARM_NODE, /* dirty segments assigned as warm node logs */
    DIRTY_COLD_NODE, /* dirty segments assigned as cold node logs */
    DIRTY,           /* to count # of dirty segments */
    PRE,             /* to count # of entirely obsolete segments */
    NR_DIRTY_TYPE,
}
