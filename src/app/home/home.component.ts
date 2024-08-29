import { AfterViewInit, Component, ElementRef, HostListener, OnDestroy, ViewChild } from '@angular/core';
import { Router } from '@angular/router';
import { AllowIn, ShortcutInput } from 'ng-keyboard-shortcuts';
import { Subscription, debounceTime, distinctUntilChanged, fromEvent, map } from 'rxjs';
import { MemoryService } from '../memory.service';
import { Channel } from '../models/channel';
import { ViewMode } from '../models/viewMode';
import { MediaType } from '../models/mediaType';
import { ToastrService } from 'ngx-toastr';
import { FocusArea, FocusAreaPrefix } from '../models/focusArea';
import { invoke } from '@tauri-apps/api/core';
import { Source } from '../models/source';
import { Filters } from '../models/filters';
import { SourceType } from '../models/sourceType';
import { animate, style, transition, trigger } from '@angular/animations';

@Component({
  selector: 'app-home',
  templateUrl: './home.component.html',
  styleUrl: './home.component.css',
  animations: [
    trigger('fadeInOut', [
      transition(':enter', [
        style({ opacity: 0 }),
        animate('500ms', style({ opacity: 1 }))
      ]),
      transition(':leave', [
        style({ opacity: 1, height: '*', padding: '*', margin: '*' }),
        animate('500ms', style({ opacity: 0, height: 0, padding: '0', margin: '0' }))
      ])
    ])
  ]
})
export class HomeComponent implements AfterViewInit, OnDestroy {
  channels: Channel[] = [];
  readonly viewModeEnum = ViewMode;
  readonly mediaTypeEnum = MediaType;
  @ViewChild('search') search!: ElementRef;
  shortcuts: ShortcutInput[] = [];
  focus: number = 0;
  focusArea = FocusArea.Tiles;
  currentWindowSize: number = window.innerWidth;
  subscriptions: Subscription[] = [];
  filters?: Filters;
  chkLiveStream = true;
  chkMovie = true;
  chkSerie = true;
  current_series_name?: string;
  current_group_name?: string;
  reachedMax = false;
  readonly PAGE_SIZE = 36;
  channelsAnim = false;
  animationTimeout: any;
  channelsAnimOut = false;

  constructor(private router: Router, public memory: MemoryService, public toast: ToastrService) {
    this.getSources();
  }

  getSources() {
    invoke("get_sources").then(sources => {
      this.memory.Sources = sources as Source[];
      if (this.memory.Sources.length == 0)
        this.reset();
      else {
        this.filters = {
          source_ids: this.memory.Sources.map(x => x.id!),
          view_type: ViewMode.All,
          media_types: [MediaType.livestream, MediaType.movie, MediaType.serie],
          page: 1
        }
        this.chkSerie = this.anyXtream();
        this.load().then(_ => _);
      }
    })
      .catch(e => {
        console.error(e);
        this.reset();
      })
  }

  reset() {
    this.router.navigateByUrl("setup");
  }

  async addEvents() {
    this.subscriptions.push(this.memory.SetGroupNode.subscribe(async idName => {
      this.clearSearch();
      this.filters!.group_id = idName.id;
      this.current_group_name = idName.name;
      await this.load();
    }));
    this.subscriptions.push(this.memory.RefreshFavs.subscribe(_ => {
      if (this.filters?.view_type == ViewMode.Favorites)
        this.load();
    }));
  }

  clearSearch() {
    this.search.nativeElement.value = "";
    this.filters!.query = "";
  }

  async loadMore() {
    this.filters!.page++;
    this.load(true);
  }

  async load(more = false) {
    if (!more) {
      this.playAnims();
    }
    try {
      let channels: Channel[] = await invoke('search', { filters: this.filters });
      if (!more) {
        this.channels = channels;
      }
      else {
        this.channels = this.channels.concat(channels);
      }
      if (channels.length < this.PAGE_SIZE)
        this.reachedMax = true;
    }
    catch (e) {
      console.error(e);
    }
  }

  playAnims() {
    this.channelsAnim = false;
    setTimeout(() => {
      this.channelsAnim = true;
      clearTimeout(this.animationTimeout);
      this.animationTimeout = setTimeout(() => this.channelsAnim = false, 500);
    }, 0)
  }

  // @HostListener('window:scroll', ['$event'])
  // async scroll(event: any) {
  //   if (window.document.documentElement.scrollHeight - (window.scrollY + window.document.documentElement.offsetHeight) == 0) {
  //     await this.loadMore();
  //   }
  // }

  // @HostListener('window:resize', ['$event'])
  // onResize(event: any) {
  //   this.currentWindowSize = event.target.innerWidth;
  // }

  ngAfterViewInit(): void {
    this.addEvents().then(_ => _);
    this.subscriptions.push(
      fromEvent(this.search.nativeElement, 'keyup').pipe(
        map((event: any) => {
          return event.target.value;
        })
        , debounceTime(300)
        , distinctUntilChanged()
      ).subscribe(async (term: string) => {
        this.focus = 0;
        this.filters!.query = term;
        this.filters!.page = 1;
        this.reachedMax = false;
        await this.load();
      }));

    this.shortcuts.push(
      {
        key: ["ctrl + f", "ctrl + space"],
        label: "Search",
        description: "Go to search",
        preventDefault: true,
        allowIn: [AllowIn.Input],
        command: _ => this.focusSearch()
      },
      {
        key: "ctrl + a",
        label: "Switching modes",
        description: "Selects the all channels mode",
        preventDefault: true,
        command: async _ => await this.switchMode(this.viewModeEnum.All)
      },
      {
        key: "ctrl + s",
        label: "Switching modes",
        description: "Selects the categories channels mode",
        allowIn: [AllowIn.Input],
        command: async _ => await this.switchMode(this.viewModeEnum.Categories)
      },
      {
        key: "ctrl + d",
        label: "Switching modes",
        description: "Selects the favorites channels mode",
        allowIn: [AllowIn.Input],
        command: async _ => await this.switchMode(this.viewModeEnum.Favorites)
      },
      {
        key: "ctrl + q",
        label: "Media viewModeEnumType Filters",
        description: "Enable/Disable livestreams",
        preventDefault: true,
        allowIn: [AllowIn.Input],
        command: async _ => {
          this.chkLiveStream = !this.chkLiveStream
          this.updateMediaTypes(MediaType.livestream)
          await this.load();
        }
      },
      {
        key: "ctrl + w",
        label: "Media Type Filters",
        description: "Enable/Disable movies",
        preventDefault: true,
        allowIn: [AllowIn.Input],
        command: async _ => {
          this.chkMovie = !this.chkMovie
          this.updateMediaTypes(MediaType.movie)
          await this.load();
        }
      },
      {
        key: "ctrl + e",
        label: "Media Type Filters",
        description: "Enable/Disable series",
        preventDefault: true,
        allowIn: [AllowIn.Input],
        command: async _ => {
          this.chkSerie = !this.chkSerie
          this.updateMediaTypes(MediaType.serie)
          await this.load();
        }
      },
      {
        key: "left",
        label: "Navigation",
        description: "Go left",
        allowIn: [AllowIn.Input],
        command: async _ => await this.nav("ArrowLeft")
      },
      {
        key: "shift + tab",
        label: "Navigation",
        description: "Go left",
        allowIn: [AllowIn.Input],
        preventDefault: true,
        command: async _ => await this.nav("ShiftTab")
      },
      {
        key: "right",
        label: "Navigation",
        description: "Go right",
        allowIn: [AllowIn.Input],
        command: async _ => await this.nav("ArrowRight")
      },
      {
        key: "tab",
        label: "Navigation",
        description: "Go right",
        allowIn: [AllowIn.Input],
        preventDefault: true,
        command: async _ => await this.nav("Tab")
      },
      {
        key: "up",
        label: "Navigation",
        description: "Go up",
        allowIn: [AllowIn.Input],
        preventDefault: true,
        command: async _ => await this.nav("ArrowUp")
      },
      {
        key: "down",
        label: "Navigation",
        description: "Go down",
        allowIn: [AllowIn.Input],
        preventDefault: true,
        command: async _ => await this.nav("ArrowDown")
      },
      {
        key: "esc",
        label: "Navigation",
        description: "Go back",
        preventDefault: true,
        allowIn: [AllowIn.Input],
        command: _ => console.log("working")
      },
      {
        key: "backspace",
        label: "Navigation",
        description: "Go back",
        command: _ => this.goBackHotkey()
      }
    );
  }

  updateMediaTypes(mediaType: MediaType) {
    let index = this.filters!.media_types.indexOf(mediaType)
    if (index == -1)
      this.filters!.media_types.push(mediaType);
    else
      this.filters!.media_types = this.filters!.media_types.splice(index, 1);
  }

  filtersVisible() {
    return this.filters?.view_type != this.viewModeEnum.Categories;
  }

  async switchMode(viewMode: ViewMode) {
    if (viewMode == this.filters?.view_type)
      return;
    this.filters!.page = 1;
    this.filters!.series_id = undefined;
    this.filters!.group_id = undefined;
    this.reachedMax = false;
    this.filters!.view_type = viewMode;
    this.clearSearch();
    await this.load();
  }

  searchFocused(): boolean {
    return document.activeElement?.id == "search";
  }

  focusSearch() {
    if (this.searchFocused()) {
      this.selectFirstChannel();
      return;
    }
    this.focus = 0;
    window.scrollTo({ top: 0, behavior: 'smooth' });
    this.search.nativeElement.focus({
      preventScroll: true
    });
  }

  goBackHotkey() {
    if (this.filters?.group_id || this.filters?.series_id) {
      if (this.filters.group_id && this.focusArea == FocusArea.Filters) {
        this.focusArea = FocusArea.Tiles;
        this.focus = 0;
      }
      this.goBack();
    }
    this.closeContextMenu();
    this.selectFirstChannel();
  }

  async goBack() {
    if (this.filters?.series_id)
      this.filters!.series_id = undefined;
    else {
      this.filters!.group_id = undefined;
    }
    this.filters!.page = 1;
    this.clearSearch();
    await this.load();
  }

  openSettings() {
    this.router.navigateByUrl("settings");
  }

  async nav(key: string) {
    let lowSize = this.currentWindowSize < 768
    if (this.memory.currentContextMenu?.menuOpen) {
      return;
    }
    let tmpFocus = 0;
    switch (key) {
      case "ArrowUp":
        tmpFocus -= 3;
        break;
      case "ArrowDown":
        tmpFocus += 3;
        break;
      case "ShiftTab":
      case "ArrowLeft":
        tmpFocus -= 1;
        break;
      case "Tab":
      case "ArrowRight":
        tmpFocus += 1;
        break;
    }
    let goOverSize = this.shortFiltersMode() ? 1 : 2;
    if (lowSize && (tmpFocus % 3 == 0) && this.focusArea == FocusArea.Tiles)
      tmpFocus / 3;
    tmpFocus += this.focus;
    if (tmpFocus < 0) {
      this.changeFocusArea(false);
    }
    else if (tmpFocus > goOverSize && this.focusArea != FocusArea.Tiles) {
      this.changeFocusArea(true);
    }
    else if (this.focusArea == FocusArea.Tiles && tmpFocus >= this.filters!.page * 36)
      await this.loadMore();
    else {
      if (tmpFocus >= this.channels.length && this.focusArea == FocusArea.Tiles)
        tmpFocus = (this.channels.length == 0 ? 1 : this.channels.length) - 1;
      this.focus = tmpFocus;
      document.getElementById(`${FocusAreaPrefix[this.focusArea]}${this.focus}`)?.focus();
    }
  }

  shortFiltersMode() {
    return !this.memory.Sources && this.focusArea == FocusArea.Filters;
  }

  anyXtream() {
    return this.memory.Sources.findIndex(x => x.source_type == SourceType.Xtream) != -1;
  }

  changeFocusArea(down: boolean) {
    let increment = down ? 1 : -1;
    this.focusArea += increment;
    if (this.focusArea == FocusArea.Filters && !this.filtersVisible())
      this.focusArea += increment
    if (this.focusArea < 0)
      this.focusArea = 0;
    this.applyFocusArea(down);
  }

  applyFocusArea(down: boolean) {
    this.focus = down ? 0 : (this.shortFiltersMode() ? 1 : 2)
    let id = FocusAreaPrefix[this.focusArea] + this.focus;
    document.getElementById(id)?.focus();
  }

  //Temporary solution because the ng-keyboard-shortcuts library doesn't seem to support ESC
  @HostListener('document:keydown', ['$event'])
  onKeyDown(event: KeyboardEvent) {
    if (event.key == "Escape" || event.key == "BrowserBack")
      this.goBackHotkey();
    if (event.key == "Enter" && this.focusArea == FocusArea.Filters)
      (document.activeElement as any).click();
  }

  selectFirstChannel() {
    this.focusArea = FocusArea.Tiles;
    (document.getElementById('first')?.firstChild as HTMLElement)?.focus();
  }

  closeContextMenu() {
    if (this.memory.currentContextMenu?.menuOpen) {
      this.memory.currentContextMenu?.closeMenu();
    }
  }

  ngOnDestroy() {
    this.subscriptions.forEach(x => x.unsubscribe());
  }
}
