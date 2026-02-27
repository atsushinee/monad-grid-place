-- 添加 pixel_index 列用于快速查询
ALTER TABLE grid_cells
ADD COLUMN IF NOT EXISTS pixel_index BIGINT GENERATED ALWAYS AS (y * 1000 + x) STORED;

-- 创建索引加速查询
CREATE INDEX IF NOT EXISTS idx_grid_cells_pixel_index ON grid_cells(pixel_index);

-- 创建快照历史表（用于回放和审计）
CREATE TABLE IF NOT EXISTS snapshot_history (
    id SERIAL PRIMARY KEY,
    owner VARCHAR(42) NOT NULL,
    cid VARCHAR(255) NOT NULL UNIQUE,
    cid_hash VARCHAR(66) NOT NULL UNIQUE,
    pixel_count INTEGER NOT NULL,
    total_price VARCHAR(50) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    tx_hash VARCHAR(66),
    block_number BIGINT
);

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_snapshot_history_owner ON snapshot_history(owner);
CREATE INDEX IF NOT EXISTS idx_snapshot_history_created_at ON snapshot_history(created_at);

-- 添加注释
COMMENT ON TABLE snapshot_history IS '存储所有 IPFS 快照的历史记录，用于回放和审计';
