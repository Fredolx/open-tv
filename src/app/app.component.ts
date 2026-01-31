import { Component, HostListener, OnInit } from '@angular/core';
import { DownloadService } from './download.service';
import { invoke } from '@tauri-apps/api/core';
import { Settings } from './models/settings';

@Component({
  selector: 'app-root',
  templateUrl: './app.component.html',
  styleUrl: './app.component.css',
})
export class AppComponent implements OnInit {
  title = 'open-tv';

  constructor(private download: DownloadService) {}

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
