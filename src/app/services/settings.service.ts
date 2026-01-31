import { Injectable } from '@angular/core';
import { TauriService } from './tauri.service';
import { Settings } from '../models/settings';

@Injectable({
  providedIn: 'root',
})
export class SettingsService {
  constructor(private tauri: TauriService) {}

  async getSettings(): Promise<Settings> {
    return await this.tauri.call<Settings>('get_settings');
  }

  async updateSettings(settings: Settings): Promise<void> {
    return await this.tauri.call<void>('update_settings', { settings });
  }

  async isContainer(): Promise<boolean> {
    return await this.tauri.call<boolean>('is_container');
  }
}
