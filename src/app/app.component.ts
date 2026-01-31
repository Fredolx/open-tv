/**
 * Beats TV - Premium IPTV Player
 * Copyright (C) 2026 Beats TV Team
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; either version 2 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License along
 * with this program; if not, write to the Free Software Foundation, Inc.,
 * 51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.
 *
 * This project is a fork of Open TV by Fredolx.
 */

import { Component, HostListener, OnInit } from '@angular/core';
import { DownloadService } from './download.service';
import { MemoryService } from './memory.service';
import { Settings } from './models/settings';
import { TauriService } from './services/tauri.service';

@Component({
  selector: 'app-root',
  templateUrl: './app.component.html',
  styleUrl: './app.component.css',
})
export class AppComponent implements OnInit {
  title = 'open-tv';

  constructor(
    private download: DownloadService,
    public memory: MemoryService,
    private tauri: TauriService,
  ) {}

  ngOnInit(): void {
    console.log('[AppComponent] ngOnInit - App starting...');
    this.applyTheme();

    // Listen for progress updates from backend via TauriService
    this.tauri.on<string>('refresh-progress', (payload) => {
      try {
        const data = JSON.parse(payload);
        this.memory.RefreshPlaylist = data.playlist;
        this.memory.RefreshActivity = data.activity;
        this.memory.RefreshPercent = data.percent;
      } catch {
        this.memory.RefreshActivity = payload;
      }
    });

    this.tauri.on<void>('refresh-start', () => {
      this.memory.IsRefreshing = true;
      this.memory.RefreshPercent = 0;
    });

    this.tauri.on<void>('refresh-end', () => {
      this.memory.IsRefreshing = false;
      this.memory.RefreshPlaylist = '';
      this.memory.RefreshActivity = '';
      this.memory.RefreshPercent = 0;
    });
  }

  applyTheme(): void {
    this.tauri
      .call<Settings>('get_settings')
      .then((settings) => {
        console.log('Settings loaded:', settings);
        // Force default theme for debugging if needed, or just log
        const themeId = settings.theme ?? 0;
        console.log('Applying theme:', themeId);
        console.log('[AppComponent] applying theme id:', themeId);
        const themeClasses = ['theme-clay-mation', 'theme-smooth-glass', 'theme-matrix-terminal'];
        document.body.classList.remove(...themeClasses);
        if (themeId >= 0 && themeId < themeClasses.length) {
          document.body.classList.add(themeClasses[themeId]);
          console.log('[AppComponent] Added class:', themeClasses[themeId]);
        } else {
          document.body.classList.add(themeClasses[0]);
          console.warn('[AppComponent] Invalid theme ID, fallback to default:', themeClasses[0]);
        }
      })
      .catch((err) => console.error('[AppComponent] Error getting settings for theme:', err));
  }

  @HostListener('document:contextmenu', ['$event'])
  onRightClick(event: MouseEvent) {
    const target = event.target as HTMLElement;
    if (this.isInsideMenuTrigger(target)) {
      return;
    }
    event.preventDefault();
  }

  private isInsideMenuTrigger(element: HTMLElement): boolean {
    return !!element.closest('[mat-menu-trigger-for], [matMenuTriggerFor]');
  }

  showDownloadManager() {
    return this.download.Downloads.size > 0;
  }
}
