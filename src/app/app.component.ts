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
import { invoke } from '@tauri-apps/api/core';
import { Settings } from './models/settings';

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
  ) {}

  ngOnInit(): void {
    this.applyTheme();
  }

  applyTheme(): void {
    invoke('get_settings').then((result) => {
      const settings = result as Settings;
      const themeId = settings.theme ?? 0;
      const themeClasses = ['theme-clay-mation', 'theme-smooth-glass', 'theme-matrix-terminal'];
      document.body.classList.remove(...themeClasses);
      if (themeId >= 0 && themeId < themeClasses.length) {
        document.body.classList.add(themeClasses[themeId]);
      }
    });
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
