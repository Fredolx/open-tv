import { Injectable } from '@angular/core';
import { TauriService } from './tauri.service';

@Injectable({
  providedIn: 'root',
})
export class EpgService {
  constructor(private tauri: TauriService) {}

  async getEpgIds(): Promise<string[]> {
    return await this.tauri.call<string[]>('get_epg_ids');
  }

  async setEpgActive(id: string, active: boolean): Promise<void> {
    return await this.tauri.call<void>('set_epg_active', { id, active });
  }
}
