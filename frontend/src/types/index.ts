export interface GridCell {
  id: number;
  x: number;
  y: number;
  color: string;
  owner: string;
  ipfs_cid: string;
  created_at: string; // Dates are serialized as strings
  updated_at: string;
}
