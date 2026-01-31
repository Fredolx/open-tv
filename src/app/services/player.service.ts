import { Injectable } from '@angular/core';
import { TauriService } from './tauri.service';
import { Channel } from '../models/channel';

@Injectable({
  providedIn: 'root',
})
export class PlayerService {
  constructor(private tauri: TauriService) {}

  async play(channel: Channel, record: boolean = false, recordPath?: string): Promise<void> {
    return await this.tauri.call<void>('play', { channel, record, recordPath });
  }

  async addLastWatched(id: number): Promise<void> {
    return await this.tauri.call<void>('add_last_watched', { id });
  }
}
