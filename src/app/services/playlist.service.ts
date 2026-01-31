import { Injectable } from '@angular/core';
import { TauriService } from './tauri.service';
import { Filters } from '../models/filters';
import { Channel } from '../models/channel';

@Injectable({
  providedIn: 'root',
})
export class PlaylistService {
  constructor(private tauri: TauriService) {}

  async loadChannels(filters: Filters): Promise<Channel[]> {
    return await this.tauri.call<Channel[]>('load_channels', { filters });
  }

  async refreshAll(): Promise<void> {
    console.log('[PlaylistService] refreshAll - Starting playlist refresh...');
    try {
      await this.tauri.call<void>('refresh_all');
      console.log('[PlaylistService] refreshAll - Refresh completed successfully.');
    } catch (e) {
      console.error('[PlaylistService] refreshAll - Error refreshing playlists:', e);
      throw e;
    }
  }

  async checkEpgOnStart(): Promise<void> {
    return await this.tauri.call<void>('on_start_check_epg');
  }

  async bulkUpdate(filters: any, action: number): Promise<void> {
    return await this.tauri.call<void>('bulk_update', { filters, action });
  }

  async hideChannel(id: number, hidden: boolean): Promise<void> {
    return await this.tauri.call<void>('hide_channel', { id, hidden });
  }

  async favoriteChannel(channelId: number): Promise<void> {
    return await this.tauri.call<void>('favorite_channel', { channelId });
  }

  async unfavoriteChannel(channelId: number): Promise<void> {
    return await this.tauri.call<void>('unfavorite_channel', { channelId });
  }
}
