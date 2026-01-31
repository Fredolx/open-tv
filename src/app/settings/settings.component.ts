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

import { Component, ElementRef, HostListener, ViewChild, NgZone } from '@angular/core';
import { debounceTime, distinctUntilChanged, fromEvent, map, Subscription } from 'rxjs';
import { Settings } from '../models/settings';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { Router } from '@angular/router';
import { open } from '@tauri-apps/plugin-dialog';
import { Source } from '../models/source';
import { MemoryService } from '../memory.service';
import { ViewMode } from '../models/viewMode';
import { NgbModal } from '@ng-bootstrap/ng-bootstrap';
import { ConfirmDeleteModalComponent } from '../confirm-delete-modal/confirm-delete-modal.component';
import { ToastrService } from 'ngx-toastr';
import { MatDialog } from '@angular/material/dialog';
import { SORT_TYPES, SortType, getSortTypeText } from '../models/sortType';
import { Tag } from '../models/tag';
import { ErrorService } from '../error.service';

@Component({
  selector: 'app-settings',
  templateUrl: './settings.component.html',
  styleUrl: './settings.component.css',
})
export class SettingsComponent {
  subscriptions: Subscription[] = [];
  activeTab = 'general';
  tags: Tag[] = [];
  settings: Settings = {
    use_stream_caching: true,
    default_view: ViewMode.All,
    volume: 100,
    restream_port: 3000,
    enable_tray_icon: true,
    zoom: 100,
    default_sort: SortType.provider,
    enable_hwdec: true,
    always_ask_save: false,
    enable_gpu: false,
  };
  viewModeEnum = ViewMode;
  sources: Source[] = [];
  sortTypes = SORT_TYPES;
  themeOptions = [
    { value: 0, label: 'Clay-Mation' },
    { value: 1, label: 'Smooth Glass' },
    { value: 2, label: 'Matrix Terminal' },
  ];

  mpvPresets = [
    { value: 'custom', label: 'Custom (Manual)' },
    { value: 'default', label: 'Default (Clean)' },
    { value: 'enhanced', label: 'Enhanced (SHARP/Pro)' },
    { value: 'stable', label: 'Stable (Anti-Repeat/Fixed)' },
    { value: 'performance', label: 'Performance+ (Smooth Motion/Low-End)' },
  ];
  selectedPreset = 'custom';
  dependencyResult: any = null;
  installStatus: any = null;
  installingMap = new Map<string, boolean>();

  @ViewChild('mpvParams') mpvParams!: ElementRef;

  constructor(
    private router: Router,
    public memory: MemoryService,
    private nav: Router,
    private modal: NgbModal,
    private toastr: ToastrService,
    public dialog: MatDialog,
    public error: ErrorService,
    private ngZone: NgZone,
  ) {
    this.checkDependencies();
    this.listenToInstallStatus();
  }

  _getSortTypeText(sortType: SortType) {
    return getSortTypeText(sortType);
  }

  isInputFocused(): boolean {
    const activeElement = document.activeElement;
    return (
      activeElement instanceof HTMLInputElement ||
      activeElement instanceof HTMLTextAreaElement ||
      activeElement instanceof HTMLSelectElement
    );
  }

  @HostListener('document:keydown', ['$event'])
  onKeyDown(event: KeyboardEvent) {
    if (
      event.key == 'Escape' ||
      event.key == 'BrowserBack' ||
      (event.key == 'Backspace' && !this.isInputFocused())
    ) {
      if (this.memory.ModalRef) {
        this.memory.ModalRef.close('close');
      } else {
        this.goBack();
      }
      event.preventDefault();
    }
  }

  ngOnInit(): void {
    this.getSettings();
    this.getSources();
  }

  getSettings() {
    invoke('get_settings').then((x) => {
      this.settings = x as Settings;
      if (this.settings.use_stream_caching == undefined) this.settings.use_stream_caching = true;
      if (this.settings.default_view == undefined) this.settings.default_view = ViewMode.All;
      if (this.settings.volume == undefined) this.settings.volume = 100;
      if (this.settings.restream_port == undefined) this.settings.restream_port = 3000;
      if (this.settings.enable_tray_icon == undefined) this.settings.enable_tray_icon = true;
      if (this.settings.zoom == undefined) this.settings.zoom = 100;
      if (this.settings.default_sort == undefined) this.settings.default_sort = SortType.provider;
      if (this.settings.enable_hwdec == undefined) this.settings.enable_hwdec = true;
      if (this.settings.always_ask_save == undefined) this.settings.always_ask_save = false;
      if (this.settings.enable_gpu == undefined) this.settings.enable_gpu = false;
      if (this.settings.max_text_lines == undefined) this.settings.max_text_lines = 2;
      if (this.settings.compact_mode == undefined) this.settings.compact_mode = false;
      if (this.settings.refresh_interval == undefined) this.settings.refresh_interval = 0;
      if (this.settings.theme == undefined) this.settings.theme = 0;

      // Auto-detect preset
      if (!this.settings.mpv_params || this.settings.mpv_params.trim() === '') {
        this.selectedPreset = 'performance';
        this.applyPreset();
      } else {
        // Try to match stable or enhanced
        Promise.all([
          invoke('get_mpv_preset', { preset: 'stable' }),
          invoke('get_mpv_preset', { preset: 'enhanced' }),
          invoke('get_mpv_preset', { preset: 'performance' }),
        ]).then(([stable, enhanced, performance]) => {
          if (this.settings.mpv_params === stable) {
            this.selectedPreset = 'stable';
          } else if (this.settings.mpv_params === enhanced) {
            this.selectedPreset = 'enhanced';
          } else if (this.settings.mpv_params === performance) {
            this.selectedPreset = 'performance';
          } else {
            // Default to custom if we can't match known presets
            this.selectedPreset = 'custom';
          }
        });
      }

      this.applyTheme(this.settings.theme);
    });
    this.getSources();
    this.getTags();
  }

  applyTheme(themeId: number) {
    const themeClasses = ['theme-clay-mation', 'theme-smooth-glass', 'theme-matrix-terminal'];
    document.body.classList.remove(...themeClasses);
    if (themeId >= 0 && themeId < themeClasses.length) {
      document.body.classList.add(themeClasses[themeId]);
    }
  }

  onThemeChange() {
    this.applyTheme(this.settings.theme ?? 0);
    this.updateSettings();
  }

  getTags() {
    invoke('detect_tags').then((tags) => {
      let t = tags as Tag[];

      this.tags = t.sort((a, b) => {
        const aVisible = a.hidden_count < a.count;
        const bVisible = b.hidden_count < b.count;

        if (aVisible && !bVisible) return -1;
        if (!aVisible && bVisible) return 1;

        const priorityRegex = /^(USA|US|United States|English|EN)$/i;
        const aPriority = priorityRegex.test(a.name);
        const bPriority = priorityRegex.test(b.name);

        if (aPriority && !bPriority) return -1;
        if (!aPriority && bPriority) return 1;

        return a.name.localeCompare(b.name);
      });
    });
  }

  toggleTag(tag: Tag, event: any) {
    const visible = event.target.checked;
    tag.hidden_count = visible ? 0 : tag.count;

    invoke('set_tag_visibility', { tag: tag.name, visible: visible }).then((count) => {
      this.toastr.success(`Updated visibility for ${count} channels`);
      this.getTags();
    });
  }

  contentTypeFilter: 'all' | 'live' | 'vod' | 'series' = 'all';

  get filteredTags() {
    if (this.contentTypeFilter === 'all') {
      return this.tags;
    }
    return this.tags.filter((tag) => {
      const t = tag as any;
      if (this.contentTypeFilter === 'live') return t.count_live > 0;
      if (this.contentTypeFilter === 'vod') return t.count_vod > 0;
      if (this.contentTypeFilter === 'series') return t.count_series > 0;
      return true;
    });
  }

  updateContentTypeFilter(type: 'all' | 'live' | 'vod' | 'series') {
    this.contentTypeFilter = type;
  }

  selectAllTags() {
    this.bulkToggleTags(true);
  }

  deselectAllTags() {
    this.bulkToggleTags(false);
  }

  bulkToggleTags(visible: boolean) {
    const tagNames = this.filteredTags.map((t) => t.name);
    invoke('set_bulk_tag_visibility', { tags: tagNames, visible: visible })
      .then((count) => {
        this.toastr.success(`Updated visibility for ${count} tags`);
        this.getTags();
      })
      .catch((e) => {
        console.error('Bulk update failed, trying individual', e);
        tagNames.forEach((name) => {
          invoke('set_tag_visibility', { tag: name, visible: visible });
        });
        this.getTags();
      });
  }

  refreshIntervals = [
    { value: 0, label: 'Disabled' },
    { value: 1, label: 'Hourly' },
    { value: 3, label: 'Every 3 Hours' },
    { value: 6, label: 'Every 6 Hours' },
    { value: 12, label: 'Every 12 Hours' },
    { value: 24, label: 'Every 24 Hours' },
  ];

  getSources() {
    invoke('get_sources').then((x) => {
      this.sources = x as Source[];
      if (this.sources.length == 0) {
        this.memory.AddingAdditionalSource = false;
        this.nav.navigateByUrl('setup');
      }
    });
  }

  ngAfterViewInit(): void {
    this.subscriptions.push(
      fromEvent(this.mpvParams.nativeElement, 'keyup')
        .pipe(
          map((event: any) => {
            return event.target.value;
          }),
          debounceTime(500),
          distinctUntilChanged(),
        )
        .subscribe(async () => {
          await this.updateSettings();
        }),
    );
    this.subscriptions.push(
      this.memory.RefreshSources.subscribe((_) => {
        this.getSources();
      }),
    );
  }

  addSource() {
    this.memory.AddingAdditionalSource = true;
    this.nav.navigateByUrl('setup');
  }

  async refreshAll() {
    console.log('refreshAll called, setting IsRefreshing = true');
    this.memory.IsRefreshing = true;
    this.memory.SeriesRefreshed.clear();

    // Get sources for progress tracking
    const sources = this.sources;
    this.memory.RefreshTotal = sources.length;
    this.memory.RefreshCurrent = 0;
    this.memory.RefreshActivity = 'Starting refresh...';
    this.memory.RefreshPlaylist = 'All Sources';
    this.memory.RefreshPercent = 0;

    try {
      for (let i = 0; i < sources.length; i++) {
        const source = sources[i];
        this.memory.RefreshCurrent = i + 1;
        this.memory.RefreshPlaylist = source.name || 'Source';
        this.memory.RefreshActivity = `Refeshing ${source.name}...`;
        this.memory.RefreshPercent = 0;

        try {
          await invoke('refresh_source', { source: source });
        } catch (e) {
          console.error(`Failed to refresh source ${source.name}:`, e);
        }
      }
      this.toastr.success('Successfully updated all sources');
    } catch (e) {
      this.error.handleError(e, 'Failed to refresh sources');
    } finally {
      console.log('refreshAll complete, setting IsRefreshing = false');
      this.memory.IsRefreshing = false;
      this.memory.RefreshPlaylist = '';
      this.memory.RefreshActivity = '';
      this.memory.RefreshPercent = 0;
      this.memory.RefreshTotal = 0;
      this.memory.RefreshCurrent = 0;
    }
  }

  async goBack() {
    await this.updateSettings();
    this.router.navigateByUrl('');
  }

  async updateSettings() {
    this.settings.mpv_params = this.settings.mpv_params?.trim();
    if (this.settings.mpv_params == '') this.settings.mpv_params = undefined;
    await invoke('update_settings', { settings: this.settings });
    this.toastr.success('Settings saved', '', {
      timeOut: 1000,
      positionClass: 'toast-bottom-right',
    });
  }

  async selectFolder() {
    const folder = await open({
      multiple: false,
      directory: true,
      canCreateDirectories: true,
    });
    if (folder) {
      this.settings.recording_path = folder;
      await this.updateSettings();
    }
  }

  async nuke() {
    this.memory.ModalRef = this.modal.open(ConfirmDeleteModalComponent, {
      backdrop: 'static',
      size: 'xl',
      keyboard: false,
    });
    this.memory.ModalRef.result.then((_) => (this.memory.ModalRef = undefined));
    this.memory.ModalRef.componentInstance.name = 'ConfirmDeleteModal';
  }

  async clearHistory() {
    await this.memory.tryIPC(
      'History cleared successfully',
      'Failed to clear history',
      async () => {
        await invoke('clear_history');
      },
    );
  }

  async applyPreset() {
    if (this.selectedPreset === 'custom') return;

    invoke('get_mpv_preset', { preset: this.selectedPreset }).then((params) => {
      this.settings.mpv_params = params as string;
      this.updateSettings();
    });
  }

  async checkDependencies() {
    this.dependencyResult = await invoke('check_dependencies');
  }

  async installDependency(name: string) {
    if (this.installingMap.get(name)) return;

    this.installingMap.set(name, true);
    try {
      await invoke('auto_install_dependency', { name });
      await this.checkDependencies();
      this.toastr.success(`${name} installed successfully!`, 'Success');
    } catch (error) {
      this.toastr.error(`Installation failed: ${error}`, 'Error');
    } finally {
      this.installingMap.set(name, false);
      this.installStatus = null;
    }
  }

  private async listenToInstallStatus() {
    const unlisten = await listen('install-status', (event: any) => {
      this.ngZone.run(() => {
        this.installStatus = event.payload;
      });
    });
    this.subscriptions.push({ unsubscribe: () => unlisten() } as any);
  }

  ngOnDestroy(): void {
    this.subscriptions.forEach((x) => x.unsubscribe());
  }
}
