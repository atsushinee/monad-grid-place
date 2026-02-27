export interface GridCell {
  id: number;
  x: number;
  y: number;
  owner: string;
  ipfs_cid: string;
  color?: string;
  link?: string;
  message?: string;
  pixel_index?: number;
  created_at: string;
  updated_at: string;
}

export interface DisplayPixel {
  x: number;
  y: number;
  color: string;
  link?: string;
  message?: string;
  owner?: string;
  extraData?: Record<string, any>;
}

export interface SnapshotPixel {
  index: number;
  x: number;
  y: number;
  color: string;
  link: string;
  message: string;
  timestamp: number;
  extraData?: Record<string, any>;
}

export interface SnapshotMetadata {
  name?: string;
  description?: string;
  website?: string;
  social?: Record<string, any>;
  [key: string]: any;
}

export interface SnapshotRequest {
  owner: string;
  new_pixels: Array<{
    x: number;
    y: number;
    color: string;
    link: string;
    message: string;
    extraData?: Record<string, any>;
  }>;
  metadata?: SnapshotMetadata;
}

export interface SnapshotResponse {
  cid: string;
  cid_hash: string;
  pixel_count: number;
  new_pixel_count: number;
  update_pixel_count: number;
  total_price: string;
  price_breakdown: {
    base_price: string;
    premium_price?: string;
    discount?: string;
    total: string;
  };
  contract_params: {
    function_name: string;
    cid_hash: string;
    pixel_count: string;
    value: string;
  };
}

export interface SnapshotHistory {
  id: number;
  owner: string;
  cid: string;
  cid_hash: string;
  pixel_count: number;
  total_price: string;
  created_at: string;
  tx_hash?: string;
  block_number?: number;
}
