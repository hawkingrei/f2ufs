enum mem_type {
    FREE_NIDS,    /* indicates the free nid list */
    NAT_ENTRIES,  /* indicates the cached nat entry */
    DIRTY_DENTS,  /* indicates dirty dentry pages */
    INO_ENTRIES,  /* indicates inode entries */
    EXTENT_CACHE, /* indicates extent cache */
    INMEM_PAGES,  /* indicates inmemory pages */
    BASE_CHECK,   /* check kernel status */
}

/* For flag in struct node_info */
enum node_info_flag {
    IS_CHECKPOINTED,   /* is it checkpointed before? */
    HAS_FSYNCED_INODE, /* is the inode fsynced before? */
    HAS_LAST_FSYNC,    /* has the latest node fsync mark? */
    IS_DIRTY,          /* this nat entry is dirty? */
    IS_PREALLOC,       /* nat entry is preallocated */
}
