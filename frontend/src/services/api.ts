import axios from 'axios';
import { GridCell, SnapshotRequest, SnapshotResponse, SnapshotHistory } from '../types';

const apiClient = axios.create({
  baseURL: 'http://127.0.0.1:3000',
});

/**
 * 获取网格单元格列表（分页）
 */
export const getGridCells = async (page: number = 1, pageSize: number = 5000): Promise<GridCell[]> => {
  const response = await apiClient.get(`/grid?page=${page}&page_size=${pageSize}`);
  return response.data;
};

/**
 * 获取单个网格单元格信息
 */
export const getGridCell = async (x: number, y: number): Promise<GridCell> => {
  const response = await apiClient.get(`/grid/${x}/${y}`);
  return response.data;
};

/**
 * 生成 IPFS 快照
 */
export const generateSnapshot = async (payload: SnapshotRequest): Promise<SnapshotResponse> => {
  const response = await apiClient.post('/snapshot', payload);
  return response.data;
};

/**
 * 提交涂色记录
 */
export const submitPaintArea = async (data: {
  owner: string;
  cid: string;
  cid_hash: string;
  pixel_count: number;
  total_price: string;
  tx_hash?: string;
  block_number?: number;
}): Promise<{ success: boolean; message: string; snapshot_id: number }> => {
  const response = await apiClient.post('/paint-area', data);
  return response.data;
};

/**
 * 获取快照历史
 */
export const getSnapshotHistory = async (
  owner: string,
  limit: number = 20,
  offset: number = 0
): Promise<{ snapshots: SnapshotHistory[]; total: number }> => {
  const response = await apiClient.get(
    `/snapshot-history?owner=${owner}&limit=${limit}&offset=${offset}`
  );
  return response.data;
};

/**
 * 获取玩家信息
 */
export const getPlayer = async (address: string): Promise<any> => {
  const response = await apiClient.get(`/player/${address}`);
  return response.data;
};

/**
 * 获取排行榜
 */
export const getLeaderboard = async (): Promise<any[]> => {
  const response = await apiClient.get('/leaderboard');
  return response.data;
};
