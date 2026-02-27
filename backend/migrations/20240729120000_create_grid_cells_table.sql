-- V5 Schema: The 'color' is now stored in IPFS.
-- We add 'link' and 'message' to be populated from the IPFS snapshot.
CREATE TABLE grid_cells (
    id SERIAL PRIMARY KEY,
    x INT NOT NULL,
    y INT NOT NULL,
    owner VARCHAR(42) NOT NULL,
    ipfs_cid VARCHAR(255) NOT NULL, -- This will store the Owner's Snapshot CID for this pixel
    link TEXT,
    message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(x, y)
);
