-- Add color column to grid_cells table for fast rendering
ALTER TABLE grid_cells 
ADD COLUMN color VARCHAR(7) NOT NULL DEFAULT '#000000';

-- Add index on owner for faster lookups
CREATE INDEX IF NOT EXISTS idx_grid_cells_owner ON grid_cells(owner);

-- Add index on updated_at for sorting
CREATE INDEX IF NOT EXISTS idx_grid_cells_updated_at ON grid_cells(updated_at);
