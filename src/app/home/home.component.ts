import { FilterChip } from './filter-chips/filter-chips.component';
import {
  AfterViewInit,
  Component,
  ElementRef,
  HostListener,
  OnDestroy,
  ViewChild,
} from '@angular/core';
import { Router } from '@angular/router';
import { AllowIn, ShortcutInput } from 'ng-keyboard-shortcuts';
import {
  Subscription,
  debounceTime,
  distinctUntilChanged,
  filter,
  fromEvent,
  map,
  skip,
  lastValueFrom,
  forkJoin,
  from,
} from 'rxjs';
import { MemoryService } from '../memory.service';
import { Channel } from '../models/channel';
import { ViewMode } from '../models/viewMode';
import { MediaType } from '../models/mediaType';
import { ToastrService } from 'ngx-toastr';
import { FocusArea, FocusAreaPrefix } from '../models/focusArea';
import { Source } from '../models/source';
import { Filters } from '../models/filters';
import { SourceType } from '../models/sourceType';
import { animate, state, style, transition, trigger } from '@angular/animations';
import { ErrorService } from '../error.service';
import { Settings } from '../models/settings';
import { getCurrentWebview } from '@tauri-apps/api/webview';
import { SortType } from '../models/sortType';
import { getVersion } from '@tauri-apps/api/app';

import { LAST_SEEN_VERSION } from '../models/localStorage';

import { Node } from '../models/node';
import { NodeType } from '../models/nodeType';
import { Stack } from '../models/stack';
import { BulkActionType } from '../models/bulkActionType';
import { TauriService } from '../services/tauri.service';
import { SettingsService } from '../services/settings.service';
import { PlaylistService } from '../services/playlist.service';
import { PlayerService } from '../services/player.service';

@Component({
  selector: 'app-home',
  templateUrl: './home.component.html',
  styleUrl: './home.component.css',
  animations: [
    trigger('fadeInOut', [
      transition(':enter', [
        style({ opacity: 0, height: 0, padding: '0', margin: '0' }),
        animate('250ms', style({ opacity: 1, height: '*', padding: '*', margin: '*' })),
      ]),
      transition(':leave', [
        style({ opacity: 1, height: '*', padding: '*', margin: '*' }),
        animate('250ms', style({ opacity: 0, height: 0, padding: '0', margin: '0' })),
      ]),
    ]),
    trigger('fade', [
      state(
        'visible',
        style({
          opacity: 1,
        }),
      ),
      state(
        'hidden',
        style({
          opacity: 0,
        }),
      ),
      transition('visible => hidden', [animate('250ms ease-out')]),
      transition('hidden => visible', [animate('250ms ease-in')]),
    ]),
  ],
})
export class HomeComponent implements AfterViewInit, OnDestroy {
  channels: Channel[] = [];
  readonly viewModeEnum = ViewMode;
  bulkActionType = BulkActionType;
  readonly mediaTypeEnum = MediaType;
  @ViewChild('search') search!: ElementRef;
  shortcuts: ShortcutInput[] = [];
  selectionMode: boolean = false;
  selectedChannels: Set<number> = new Set();
  focus: number = 0;
  focusArea = FocusArea.Tiles;
  viewType = ViewMode.All;
  currentWindowSize: number = window.innerWidth;
  subscriptions: Subscription[] = [];
  filters?: Filters;
  chkLiveStream = true;
  chkMovie = true;
  chkSerie = true;
  reachedMax = false;
  readonly PAGE_SIZE = 36;
  channelsVisible = true;
  prevSearchValue: String = '';
  loading = false;
  nodeStack: Stack = new Stack();
  showScrollTop = false;

  // New UI Properties
  // New UI Properties
  filterChips: FilterChip[] = [
    { id: 'live', label: 'Live TV', active: true, type: 'media', value: MediaType.livestream },
    { id: 'movies', label: 'Movies', active: false, type: 'media', value: MediaType.movie },
    { id: 'series', label: 'Series', active: false, type: 'media', value: MediaType.serie },
  ];
  genreInput: string = '';
  minRating: number = 0;
  selectedChannelForModal: Channel | null = null;
  isLoadingDetails: boolean = false;

  scrollToTop() {
    window.scrollTo({ top: 0, behavior: 'smooth' });
  }

  constructor(
    private router: Router,
    public memory: MemoryService,
    public toast: ToastrService,
    private error: ErrorService,
    private tauri: TauriService,
    private settingsService: SettingsService,
    private playlistService: PlaylistService,
    private playerService: PlayerService,
  ) {
    this.getSources();
  }

  bulkActionFromBar(action: string) {
    switch (action) {
      case 'Favorite':
        this.bulkActionOnSelected(this.bulkActionType.Favorite);
        break;
      case 'Hide':
        this.bulkActionOnSelected(this.bulkActionType.Hide);
        break;
      case 'Whitelist':
        this.whitelistSelected();
        break;
    }
  }

  getSources() {
    let get_settings = this.tauri.call('get_settings');
    let get_sources = this.tauri.call('get_sources');
    Promise.all([get_settings, get_sources])
      .then((data) => {
        let settings = data[0] as Settings;
        let sources = data[1] as Source[];
        this.memory.settings = settings;
        if (settings.zoom) getCurrentWebview().setZoom(Math.trunc(settings.zoom! * 100) / 10000);
        this.memory.trayEnabled = settings.enable_tray_icon ?? true;
        this.memory.AlwaysAskSave = settings.always_ask_save ?? false;
        this.memory.Sources = new Map(sources.filter((x) => x.enabled).map((s) => [s.id!, s]));
        if (sources.length == 0) {
          this.reset();
        } else {
          getVersion().then((version) => {
            if (localStorage.getItem(LAST_SEEN_VERSION) != version) {
              this.memory.AppVersion = version;
              this.memory.updateVersion();
            }
          });
          sources
            .filter((x) => x.source_type == SourceType.Custom)
            .map((x) => x.id!)
            .forEach((x) => this.memory.CustomSourceIds?.add(x));
          sources
            .filter((x) => x.source_type == SourceType.Xtream)
            .map((x) => x.id!)
            .forEach((x) => this.memory.XtreamSourceIds.add(x));
          if (
            this.memory.XtreamSourceIds.size > 0 &&
            !sessionStorage.getItem('epgCheckedOnStart')
          ) {
            sessionStorage.setItem('epgCheckedOnStart', 'true');
            this.playlistService.checkEpgOnStart();
          }
          this.filters = {
            source_ids: Array.from(this.memory.Sources.keys()),
            view_type: settings.default_view ?? ViewMode.All,
            media_types: [MediaType.livestream], // Default to Live TV only
            page: 1,
            use_keywords: false,
            sort: SortType.provider,
          };
          if (settings.default_sort != undefined && settings.default_sort != SortType.provider) {
            this.memory.Sort.next([settings.default_sort, false]);
            this.filters.sort = settings.default_sort;
          }

          // Default selection state
          this.chkLiveStream = true;
          this.chkMovie = false;
          this.chkSerie = false;

          // Refresh on start logic
          const refreshOnStart = settings.refresh_on_start === true;
          // Refresh interval logic
          const refreshInterval = settings.refresh_interval || 0;
          const lastRefresh = settings.last_refresh || 0;
          const now = Date.now();
          const hoursSinceLastRefresh = (now - lastRefresh) / (1000 * 60 * 60);

          let shouldRefresh = false;
          let refreshReason = '';

          if (refreshOnStart && !sessionStorage.getItem('refreshedOnStart')) {
            shouldRefresh = true;
            refreshReason = 'refresh on start enabled';
          } else if (refreshInterval > 0 && hoursSinceLastRefresh >= refreshInterval) {
            shouldRefresh = true;
            refreshReason = `interval of ${refreshInterval} hours passed`;
          }

          if (shouldRefresh) {
            sessionStorage.setItem('refreshedOnStart', 'true');
            this.refreshAll(refreshReason).then((_) => _);
          }

          this.load().then((_) => _);
        }
      })
      .catch((e) => {
        this.error.handleError(e);
        this.reset();
      });
  }

  async refreshOnStart() {
    await this.refreshAll('refresh on start enabled');
  }

  async refreshAll(reason: string = 'user requested') {
    this.toast.info(`Refreshing all sources... (${reason})`);
    try {
      await this.playlistService.refreshAll();
      this.memory.settings.last_refresh = Date.now();
      await this.settingsService.updateSettings(this.memory.settings);
      this.toast.success(`Successfully refreshed all sources (${reason})`);
    } catch (e) {
      this.error.handleError(e, `Failed to refresh all sources (${reason})`);
    }
  }

  async reload() {
    await this.load();
  }

  reset() {
    this.router.navigateByUrl('setup');
  }

  async addEvents() {
    this.subscriptions.push(
      this.memory.HideChannels.subscribe((val) => {
        this.channelsVisible = val;
      }),
    );
    this.subscriptions.push(
      this.memory.SetFocus.subscribe((focus) => {
        this.focus = focus;
      }),
    );
    this.subscriptions.push(
      this.memory.SetNode.subscribe(async (dto) => {
        this.nodeStack.add(
          new Node(dto.id, dto.name, dto.type, this.filters?.query, this.filters?.view_type),
        );
        if (dto.type == NodeType.Category) this.filters!.group_id = dto.id;
        else if (dto.type == NodeType.Series) {
          this.filters!.series_id = dto.id;
          this.filters!.source_ids = [dto.sourceId!];
        } else if (dto.type == NodeType.Season) this.filters!.season = dto.id;

        if (this.filters!.view_type == ViewMode.Hidden) {
          this.filters!.view_type = ViewMode.Categories;
        }

        this.clearSearch();
        await this.load();
        if (this.focusArea == FocusArea.Tiles) this.selectFirstChannelDelayed(100);
      }),
    );
    this.subscriptions.push(
      this.memory.Refresh.subscribe((scroll) => {
        this.load();
        if (scroll) window.scrollTo({ top: 0, behavior: 'instant' });
      }),
    );
    this.subscriptions.push(
      this.memory.Sort.pipe(skip(1)).subscribe(async ([sort, load]) => {
        if (!this.filters || !load) return;
        this.filters!.sort = sort;
        await this.load();
      }),
    );
  }

  clearSearch() {
    this.search.nativeElement.value = '';
    this.prevSearchValue = '';
    this.filters!.query = '';
  }

  async loadMore() {
    this.load(true);
  }

  async load(more = false) {
    this.loading = true;
    if (more) {
      this.filters!.page++;
    } else {
      this.filters!.page = 1;
    }
    try {
      let channels: Channel[] = await this.tauri.call<Channel[]>('search', {
        filters: this.filters,
      });

      // Fallback search logic: if no results and filter is active, try searching across all categories
      if (
        !more &&
        channels.length === 0 &&
        this.filters!.query &&
        this.filters!.media_types.length < 3
      ) {
        // Expand search to all types
        this.chkLiveStream = true;
        this.chkMovie = true;
        this.chkSerie = true;
        this.filters!.media_types = [MediaType.livestream, MediaType.movie, MediaType.serie];
        channels = await this.tauri.call<Channel[]>('search', { filters: this.filters });
        if (channels.length > 0) {
          this.toast.info('No results in current category. Found matches in other areas!');
        }
      }

      if (!more) {
        this.channels = channels;
        this.channelsVisible = true;
        // prevent flicker of hiding opacity
        this.viewType = this.filters!.view_type;
      } else {
        this.channels = this.channels.concat(channels);
      }
      this.reachedMax = channels.length < this.PAGE_SIZE;
    } catch (e) {
      this.error.handleError(e);
    }
    this.loading = false;
  }

  checkScrollTop() {
    const scrollPosition =
      window.pageYOffset || document.documentElement.scrollTop || document.body.scrollTop || 0;
    this.showScrollTop = scrollPosition > 300;
  }

  async checkScrollEnd() {
    if (this.reachedMax === true || this.loading === true) return;
    const scrollHeight = document.documentElement.scrollHeight;
    const scrollTop = window.scrollY || document.documentElement.scrollTop;
    const clientHeight = window.innerHeight || document.documentElement.clientHeight;
    if (scrollTop + clientHeight >= scrollHeight * 0.75) {
      await this.loadMore();
    }
  }

  @HostListener('window:scroll', ['$event'])
  async scroll(event: any) {
    this.checkScrollTop();
    await this.checkScrollEnd();
  }

  ngAfterViewInit(): void {
    this.addEvents().then((_) => _);
    this.subscriptions.push(
      fromEvent(this.search.nativeElement, 'keyup')
        .pipe(
          filter((event: any) => event.key !== 'Escape'),
          map((event: any) => {
            this.focus = 0;
            this.focusArea = FocusArea.Tiles;
            if (this.channelsVisible && event.target.value != this.prevSearchValue)
              this.channelsVisible = false;
            this.prevSearchValue = event.target.value;
            return event.target.value;
          }),
          debounceTime(300),
        )
        .subscribe(async (term: string) => {
          this.filters!.query = term;
          await this.load();
        }),
    );

    this.shortcuts.push(
      {
        key: ['ctrl + f', 'ctrl + space', 'cmd + f'],
        label: 'Search',
        description: 'Go to search',
        preventDefault: true,
        allowIn: [AllowIn.Input],
        command: (_) => this.focusSearch(),
      },
      {
        key: ['ctrl + a', 'cmd + a'],
        label: 'Switching modes',
        description: 'Selects the all channels view',
        preventDefault: true,
        command: async (_) => await this.switchMode(this.viewModeEnum.All),
      },
      {
        key: ['ctrl + s', 'cmd + s'],
        label: 'Switching modes',
        description: 'Selects the categories view',
        command: async (_) => await this.switchMode(this.viewModeEnum.Categories),
      },
      {
        key: ['ctrl + d', 'cmd + d'],
        label: 'Switching modes',
        description: 'Selects the history view',
        command: async (_) => await this.switchMode(this.viewModeEnum.History),
      },
      {
        key: ['ctrl + r', 'cmd + r'],
        label: 'Switching modes',
        description: 'Selects the favorites view',
        command: async (_) => await this.switchMode(this.viewModeEnum.Favorites),
      },
      {
        key: 'ctrl + q',
        label: 'Media Type Filters',
        description: 'Enable/Disable livestreams',
        preventDefault: true,
        allowIn: [AllowIn.Input],
        command: async (_) => {
          this.chkLiveStream = !this.chkLiveStream;
          this.updateMediaTypes(MediaType.livestream);
        },
      },
      {
        key: 'ctrl + w',
        label: 'Media Type Filters',
        description: 'Enable/Disable movies',
        preventDefault: true,
        allowIn: [AllowIn.Input],
        command: async (_) => {
          this.chkMovie = !this.chkMovie;
          this.updateMediaTypes(MediaType.movie);
        },
      },
      {
        key: 'ctrl + e',
        label: 'Media Type Filters',
        description: 'Enable/Disable series',
        preventDefault: true,
        allowIn: [AllowIn.Input],
        command: async (_) => {
          this.chkSerie = !this.chkSerie;
          this.updateMediaTypes(MediaType.serie);
        },
      },
      {
        key: 'left',
        label: 'Navigation',
        description: 'Go left',
        allowIn: [AllowIn.Input],
        command: async (_) => await this.nav('ArrowLeft'),
      },
      {
        key: 'right',
        label: 'Navigation',
        description: 'Go right',
        allowIn: [AllowIn.Input],
        command: async (_) => await this.nav('ArrowRight'),
      },
      {
        key: 'up',
        label: 'Navigation',
        description: 'Go up',
        allowIn: [AllowIn.Input],
        preventDefault: true,
        command: async (_) => await this.nav('ArrowUp'),
      },
      {
        key: 'down',
        label: 'Navigation',
        description: 'Go down',
        allowIn: [AllowIn.Input],
        preventDefault: true,
        command: async (_) => await this.nav('ArrowDown'),
      },
    );
  }

  updateMediaTypes(mediaType: MediaType) {
    let index = this.filters!.media_types.indexOf(mediaType);
    if (index == -1) this.filters!.media_types.push(mediaType);
    else this.filters!.media_types.splice(index, 1);
    this.load();
  }

  filtersVisible() {
    return !this.filters?.series_id;
  }

  async switchMode(viewMode: ViewMode) {
    if (viewMode == this.filters?.view_type) return;
    this.filters!.series_id = undefined;
    this.filters!.group_id = undefined;
    this.filters!.view_type = viewMode;
    this.filters!.season = undefined;
    this.clearSearch();
    this.nodeStack.clear();
    await this.load();
  }

  searchFocused(): boolean {
    return document.activeElement?.id == 'search';
  }

  focusSearch() {
    if (this.searchFocused()) {
      this.selectFirstChannel();
      return;
    } else {
      this.focus = 0;
      this.focusArea = FocusArea.Tiles;
    }
    window.scrollTo({ top: 0, behavior: 'smooth' });
    this.search.nativeElement.focus({
      preventScroll: true,
    });
  }

  async goBackHotkey() {
    if (this.memory.ModalRef) {
      if (
        this.memory.ModalRef.componentInstance.name != 'RestreamModalComponent' ||
        !this.memory.ModalRef.componentInstance.started
      )
        this.memory.ModalRef.close('close');
      return;
    } else if (this.memory.currentContextMenu?.menuOpen) {
      this.closeContextMenu();
    } else if (this.searchFocused()) {
      this.selectFirstChannel();
    } else if (this.filters?.query) {
      if (this.filters?.query) {
        this.clearSearch();
        await this.load();
      }
      this.selectFirstChannelDelayed(100);
    } else if (this.nodeStack.hasNodes()) {
      await this.goBack();
      this.selectFirstChannelDelayed(100);
    } else {
      this.selectFirstChannel();
    }
  }

  selectFirstChannelDelayed(milliseconds: number) {
    setTimeout(() => this.selectFirstChannel(), milliseconds);
  }

  async goBack() {
    var node = this.nodeStack.pop();
    if (node.type == NodeType.Category) this.filters!.group_id = undefined;
    else if (node.type == NodeType.Series) {
      this.filters!.series_id = undefined;
      this.filters!.source_ids = Array.from(this.memory.Sources.keys());
    } else if (node.type == NodeType.Season) {
      this.filters!.season = undefined;
    }
    if (node.query) {
      this.search.nativeElement.value = node.query;
      this.filters!.query = node.query;
    }
    if (node.fromViewType && this.filters!.view_type !== node.fromViewType) {
      this.filters!.view_type = node.fromViewType;
    }
    await this.load();
  }

  openSettings() {
    this.router.navigateByUrl('settings');
  }

  async nav(key: string) {
    if (this.searchFocused()) return;
    let lowSize = this.currentWindowSize < 768;
    if (this.memory.currentContextMenu?.menuOpen || this.memory.ModalRef) {
      return;
    }
    let tmpFocus = 0;
    switch (key) {
      case 'ArrowUp':
        tmpFocus -= 3;
        break;
      case 'ArrowDown':
        tmpFocus += 3;
        break;
      case 'ShiftTab':
      case 'ArrowLeft':
        tmpFocus -= 1;
        break;
      case 'Tab':
      case 'ArrowRight':
        tmpFocus += 1;
        break;
    }
    let goOverSize = this.shortFiltersMode() ? 1 : 2;
    if (lowSize && tmpFocus % 3 == 0 && this.focusArea == FocusArea.Tiles) tmpFocus / 3;
    tmpFocus += this.focus;
    if (tmpFocus < 0) {
      this.changeFocusArea(false);
    } else if (tmpFocus > goOverSize && this.focusArea == FocusArea.Filters) {
      this.changeFocusArea(true);
    } else if (tmpFocus > 4 && this.focusArea == FocusArea.ViewMode) {
      this.changeFocusArea(true);
    } else if (
      this.focusArea == FocusArea.Tiles &&
      tmpFocus >= this.filters!.page * 36 &&
      !this.reachedMax
    )
      await this.loadMore();
    else {
      if (tmpFocus >= this.channels.length && this.focusArea == FocusArea.Tiles)
        tmpFocus = (this.channels.length == 0 ? 1 : this.channels.length) - 1;
      this.focus = tmpFocus;
      setTimeout(() => {
        document.getElementById(`${FocusAreaPrefix[this.focusArea]}${this.focus}`)?.focus();
      }, 0);
    }
  }

  shortFiltersMode() {
    return this.filters?.source_ids.findIndex((x) => this.memory.XtreamSourceIds.has(x)) == -1;
  }

  anyXtream() {
    return (
      Array.from(this.memory.Sources.values()).findIndex(
        (x) => x.source_type == SourceType.Xtream,
      ) != -1
    );
  }

  changeFocusArea(down: boolean) {
    let increment = down ? 1 : -1;
    this.focusArea += increment;
    if (this.focusArea == FocusArea.Filters && !this.filtersVisible()) this.focusArea += increment;
    if (this.focusArea < 0) this.focusArea = 0;
    this.applyFocusArea(down);
  }

  /**
   * Toggles the multi-select mode.
   * When enabled, clicking channels selects them instead of playing.
   * When disabled, clears the current selection.
   */
  toggleSelectionMode() {
    this.selectionMode = !this.selectionMode;
    if (!this.selectionMode) {
      this.clearSelection();
    }
  }

  /**
   * Toggles the selection state of a specific channel.
   * @param id The ID of the channel to toggle.
   */
  toggleChannelSelection(id: number) {
    if (this.selectedChannels.has(id)) {
      this.selectedChannels.delete(id);
    } else {
      this.selectedChannels.add(id);
    }
  }

  /**
   * Clears all currently selected channels.
   */
  clearSelection() {
    this.selectedChannels.clear();
  }

  /**
   * Selects all channels currently visible in the view.
   */
  selectAllView() {
    this.channels.forEach((c) => {
      if (c.id !== undefined) {
        this.selectedChannels.add(c.id);
      }
    });
  }

  /**
   * Unhides selected channels and hides all other channels in the currently active sources.
   * This effectively "whitelists" the selection.
   */
  async whitelistSelected() {
    if (this.selectedChannels.size === 0) return;

    try {
      this.memory.Loading = true;
      const selectedIds = Array.from(this.selectedChannels);
      const sourceIds = this.filters?.source_ids || [];

      if (sourceIds.length === 0) return;

      // 1. Unhide all selected
      const unhidePromises = selectedIds.map((id) =>
        from(this.playlistService.hideChannel(id, false)),
      );

      // 2. Hide everything else in these sources
      // We'll create a special filter for 'everything else'
      const hideOthersFilter: Filters = {
        ...this.filters!,
        query: '', // All channels
        view_type: ViewMode.All,
        page: 1, // We'll handle this differently if we need more, but bulk_update usually handles query-based
      };

      // Note: The current bulk_update in Rust uses filters to identify targets.
      // To hide 'everything else', we'd ideally want a "NOT IN (selectedIds)" filter.
      // Since the backend doesn't have it, we'll hide ALL in the source, then UNHIDE the selected.

      // Step A: Hide all in sources matching general filters (Live/Movie/Serie)
      await this.playlistService.bulkUpdate(
        { ...hideOthersFilter, query: undefined },
        BulkActionType.Hide,
      );

      // Step B: Unhide the selected ones specifically
      await lastValueFrom(forkJoin(unhidePromises));

      this.toast.success(`Whitelisted ${selectedIds.length} channels. Others hidden.`);
      this.clearSelection();
      this.reload();
    } catch (e) {
      this.error.handleError(e);
    } finally {
      this.memory.Loading = false;
    }
  }

  /**
   * Executes a bulk action (Hide, Unhide, Favorite, Unfavorite) on the currently selected channels.
   * @param action The type of action to perform.
   */
  async bulkActionOnSelected(action: BulkActionType) {
    if (this.selectedChannels.size === 0) return;

    try {
      this.memory.Loading = true;
      const ids = Array.from(this.selectedChannels);

      const promises = [];
      for (const id of ids) {
        if (action === BulkActionType.Hide) {
          promises.push(from(this.playlistService.hideChannel(id, true)));
        } else if (action === BulkActionType.Unhide) {
          promises.push(from(this.playlistService.hideChannel(id, false)));
        } else if (action === BulkActionType.Favorite) {
          promises.push(from(this.playlistService.favoriteChannel(id)));
        } else if (action === BulkActionType.Unfavorite) {
          promises.push(from(this.playlistService.unfavoriteChannel(id)));
        }
      }

      await lastValueFrom(forkJoin(promises));

      this.toast.success(`Updated ${ids.length} channels`);
      this.clearSelection();
      this.reload();
    } catch (e) {
      this.error.handleError(e);
    } finally {
      this.memory.Loading = false;
    }
  }

  applyFocusArea(down: boolean) {
    this.focus = down
      ? 0
      : this.focusArea == FocusArea.Filters
        ? this.shortFiltersMode()
          ? 1
          : 2
        : 4;
    let id = FocusAreaPrefix[this.focusArea] + this.focus;
    document.getElementById(id)?.focus();
  }

  // Remove temporary ESC workaround - use proper keyboard shortcut handling
  @HostListener('document:keydown', ['$event'])
  onKeyDown(event: KeyboardEvent) {
    if (event.key == 'Tab' && !this.memory.ModalRef) {
      event.preventDefault();
      this.nav(event.shiftKey ? 'ShiftTab' : 'Tab');
    }
    if (event.key == 'Enter' && this.focusArea == FocusArea.Filters)
      (document.activeElement as any).click();
  }

  selectFirstChannel() {
    this.focusArea = FocusArea.Tiles;
    this.focus = 0;
    (document.getElementById('first')?.firstChild as HTMLElement)?.focus();
  }

  closeContextMenu() {
    if (this.memory.currentContextMenu?.menuOpen) {
      this.memory.currentContextMenu?.closeMenu();
    }
  }

  ngOnDestroy() {
    this.subscriptions.forEach((x) => x?.unsubscribe());
  }

  async toggleKeywords() {
    this.filters!.use_keywords = !this.filters!.use_keywords;
    await this.load();
  }

  async bulkAction(action: BulkActionType) {
    if (this.filters?.series_id && !this.filters?.season) {
      return;
    }
    const actionName = BulkActionType[action].toLowerCase();
    try {
      await this.playlistService.bulkUpdate(this.filters, action);
      await this.load();
      this.toast.success(`Successfully executed bulk update: ${actionName}`);
    } catch (e) {
      this.error.handleError(e);
    }
  }

  // New UI Methods

  onFilterChipChanged(chip: FilterChip) {
    chip.active = !chip.active;
    if (chip.type === 'media') {
      this.updateMediaTypes(chip.value);
      // Sync legacy properties
      if (chip.value === MediaType.livestream) this.chkLiveStream = chip.active;
      if (chip.value === MediaType.movie) this.chkMovie = chip.active;
      if (chip.value === MediaType.serie) this.chkSerie = chip.active;
    }
  }

  async toggleVods(state: boolean) {
    this.chkMovie = state;
    this.chkSerie = state;
    if (this.filters) {
      if (state) {
        this.filters.media_types = Array.from(
          new Set([...(this.filters.media_types || []), MediaType.movie, MediaType.serie]),
        );
      } else {
        this.filters.media_types = this.filters.media_types?.filter(
          (t) => t !== MediaType.movie && t !== MediaType.serie,
        );
      }
      await this.load();
    }
  }

  async updateGenre(value: string) {
    this.genreInput = value;
    if (this.filters) {
      this.filters.genre = value || undefined;
      await this.load();
    }
  }

  async updateRating(value: number) {
    this.minRating = value;
    if (this.filters) {
      this.filters.rating_min = value > 0 ? value : undefined;
      await this.load();
    }
  }

  async openDetails(channel: Channel) {
    this.selectedChannelForModal = channel;
    this.isLoadingDetails = true;
    try {
      // Simulate loading details if needed, or invoke backend
      // this.selectedChannelForModal = await invoke('get_channel_details', { id: channel.id });
    } catch (e) {
      console.error('Error opening details', e);
    } finally {
      this.isLoadingDetails = false;
    }
  }

  onModalClose() {
    this.selectedChannelForModal = null;
  }

  async onModalPlay() {
    if (this.selectedChannelForModal) {
      const channel = this.selectedChannelForModal;
      this.onModalClose();
      try {
        await this.playerService.play(channel);
        this.playerService.addLastWatched(channel.id!).catch(console.error);
      } catch (e) {
        this.error.handleError(e);
      }
    }
  }

  bulkHide(hide: boolean) {
    this.bulkActionOnSelected(hide ? BulkActionType.Hide : BulkActionType.Unhide);
  }

  bulkFavorite(fav: boolean) {
    this.bulkActionOnSelected(fav ? BulkActionType.Favorite : BulkActionType.Unfavorite);
  }
}
