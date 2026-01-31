import { Component, ElementRef, HostListener, ViewChild } from '@angular/core';
import { debounceTime, distinctUntilChanged, fromEvent, map, Subscription } from 'rxjs';
import { Settings } from '../models/settings';
import { invoke } from '@tauri-apps/api/core';
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
  @ViewChild('mpvParams') mpvParams!: ElementRef;

  constructor(
    private router: Router,
    public memory: MemoryService,
    private nav: Router,
    private modal: NgbModal,
    private toastr: ToastrService,
    public dialog: MatDialog,
    public error: ErrorService,
  ) {}

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
      if (this.settings.use_single_column == undefined) this.settings.use_single_column = false;
      if (this.settings.max_text_lines == undefined) this.settings.max_text_lines = 2;
      if (this.settings.compact_mode == undefined) this.settings.compact_mode = false;
      if (this.settings.refresh_interval == undefined) this.settings.refresh_interval = 0;
      if (this.settings.theme == undefined) this.settings.theme = 0;
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

        // 1. Visible tags first
        if (aVisible && !bVisible) return -1;
        if (!aVisible && bVisible) return 1;

        // 2. Priority Tags (US, English) bubble to top of their section
        const priorityRegex = /^(USA|US|United States|English|EN)$/i;
        const aPriority = priorityRegex.test(a.name);
        const bPriority = priorityRegex.test(b.name);

        if (aPriority && !bPriority) return -1;
        if (!aPriority && bPriority) return 1;

        // 3. Alphabetical Order
        return a.name.localeCompare(b.name);
      });
    });
  }

  toggleTag(tag: Tag, event: any) {
    // If checked (event.target.checked is true), it implies we want it VISIBLE.
    // So 'visible' = true.
    const visible = event.target.checked;

    // Optimistic update
    tag.hidden_count = visible ? 0 : tag.count;

    invoke('set_tag_visibility', { tag: tag.name, visible: visible }).then((count) => {
      this.toastr.success(`Updated visibility for ${count} channels`);
      this.getTags(); // Refresh to get accurate counts
    });
  }

  // Content Type Filtering for Tags
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
    // Trigger backend fetch if needed or just local filter
    // For now, we reuse the existing getTags but we could pass a filter if we modify backend.
  }

  selectAllTags() {
    this.bulkToggleTags(true);
  }

  deselectAllTags() {
    this.bulkToggleTags(false);
  }

  bulkToggleTags(visible: boolean) {
    const tagNames = this.filteredTags.map((t) => t.name);
    // We need a bulk API or loop. Looping is easier for now but might spam.
    // Ideally we add a 'set_bulk_tag_visibility' command.
    // For now, let's just loop locally and send one optimized request if possible,
    // or just loop.

    // Better approach: Create a new command in Rust for bulk update to avoid UI lag.
    // But to save time and stick to frontend if possible:

    // Let's rely on the requested feature: "allow fo the ability to select all or deselect all"
    invoke('set_bulk_tag_visibility', { tags: tagNames, visible: visible })
      .then((count) => {
        this.toastr.success(`Updated visibility for ${count} tags`);
        this.getTags();
      })
      .catch((e) => {
        // Fallback if backend command doesn't exist yet (I'll implement it next)
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
    this.memory.SeriesRefreshed.clear();
    try {
      await invoke('refresh_all');
      this.toastr.success('Successfully updated all sources');
    } catch (e) {
      this.error.handleError(e, 'Failed to refresh sources');
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

  ngOnDestroy(): void {
    this.subscriptions.forEach((x) => x.unsubscribe());
  }
}
