import { AfterViewInit, Component, ElementRef, HostListener, OnDestroy, ViewChild } from '@angular/core';
import { Router } from '@angular/router';
import { AllowIn, ShortcutInput } from 'ng-keyboard-shortcuts';
import { Subscription, debounceTime, distinctUntilChanged, fromEvent, map } from 'rxjs';
import { MemoryService } from '../memory.service';
import { Cache } from '../models/cache';
import { Channel } from '../models/channel';
import { ViewMode } from '../models/viewMode';
import { MediaType } from '../models/mediaType';
import { ToastrService } from 'ngx-toastr';
import { FocusArea, FocusAreaPrefix } from '../models/focusArea';

@Component({
  selector: 'app-home',
  templateUrl: './home.component.html',
  styleUrls: ['./home.component.scss']
})
export class HomeComponent implements AfterViewInit, OnDestroy {
  channels: Channel[] = [];
  favChannels: Channel[] = [];
  viewMode = ViewMode.All;
  viewModeEnum = ViewMode;
  electron: any = (window as any).electronAPI;
  lastTerm?: string;
  @ViewChild('search') search!: ElementRef;
  @ViewChild('searchFavs') searchFavs!: ElementRef;
  @ViewChild('searchCats') searchCats!: ElementRef;
  defaultElementsToRetrieve: number = 36;
  elementsToRetrieve: number = this.defaultElementsToRetrieve;
  channelsLeft: number = 0;
  shortcuts: ShortcutInput[] = [];
  chkLivestream: boolean = true;
  chkMovie: boolean = true;
  chkSerie: boolean = true;
  categories?: Array<Channel>;
  focus: number = 0;
  focusArea = FocusArea.Tiles;
  currentWindowSize: number = window.innerWidth;
  subscriptions: Subscription[] = [];

  constructor(private router: Router, public memory: MemoryService, public toast: ToastrService) {
    if (this.memory.Channels.length > 0) {
      this.getChannels();
      this.getCategories();
      this.addEvents();
    }
    else {
      this.electron.getCache().then((x: { cache: Cache, favs: Channel[], performedMigration?: boolean }) => {
        if (x.cache?.channels?.length > 0) {
          if (x.performedMigration)
            toast.info("Your channel data has been migrated. Please delete & re-load your channels if you notice any issues", undefined, { timeOut: 20000 })
          this.memory.Channels = x.cache.channels;
          if (x.cache.xtream)
            this.memory.Xtream = x.cache.xtream
          if (x.cache.url)
            this.memory.Url = x.cache.url;
          this.memory.FavChannels = x.favs;
          this.getChannels();
          this.getCategories();
          this.addEvents();
        }
        else
          router.navigateByUrl("setup");
      });
    }
  }

  addEvents() {
    this.subscriptions.push(this.memory.NeedToRefreshFavorites.subscribe(_ => {
      if (this.channels.length == 1 && this.lastTerm?.trim()) {
        this.clearSearch();
      }
      this.load();
    }));
    this.subscriptions.push(this.memory.SwitchingNode.subscribe(_ => {
      this.clearSearch();
      this.load();
    }));
  }

  clearSearch() {
    this.search.nativeElement.value = "";
    this.lastTerm = "";
  }

  loadMore() {
    this.elementsToRetrieve += 36;
    this.load();
  }

  load() {
    this.channels = this.filterChannels(this.lastTerm ?? "");
  }

  @HostListener('window:scroll', ['$event'])
  scroll(event: any) {
    if (window.innerHeight + window.scrollY - window.document.documentElement.offsetHeight == 0 && this.channelsLeft > 0) {
      this.loadMore();
    }
  }

  @HostListener('window:resize', ['$event'])
  onResize(event: any) {
    this.currentWindowSize = event.target.innerWidth;
  }

  ngAfterViewInit(): void {
    this.subscriptions.push(
      fromEvent(this.search.nativeElement, 'keyup').pipe(
        map((event: any) => {
          return event.target.value;
        })
        , debounceTime(300)
        , distinctUntilChanged()
      ).subscribe((term: string) => {
        this.focus = 0;
        this.elementsToRetrieve = this.defaultElementsToRetrieve;
        this.lastTerm = term;
        this.channels = this.filterChannels(term);
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
        command: _ => this.switchMode(this.viewModeEnum.All)
      },
      {
        key: "ctrl + s",
        label: "Switching modes",
        description: "Selects the categories channels mode",
        allowIn: [AllowIn.Input],
        command: _ => this.switchMode(this.viewModeEnum.Categories)
      },
      {
        key: "ctrl + d",
        label: "Switching modes",
        description: "Selects the favorites channels mode",
        allowIn: [AllowIn.Input],
        command: _ => this.switchMode(this.viewModeEnum.Favorites)
      },
      {
        key: "ctrl + q",
        label: "Media Type Filters",
        description: "Enable/Disable livestreams",
        preventDefault: true,
        allowIn: [AllowIn.Input],
        command: _ => {
          this.chkLivestream = !this.chkLivestream;
          this.load();
        }
      },
      {
        key: "ctrl + w",
        label: "Media Type Filters",
        description: "Enable/Disable movies",
        preventDefault: true,
        allowIn: [AllowIn.Input],
        command: _ => {
          this.chkMovie = !this.chkMovie;
          this.load();
        }
      },
      {
        key: "ctrl + e",
        label: "Media Type Filters",
        description: "Enable/Disable series",
        preventDefault: true,
        allowIn: [AllowIn.Input],
        command: _ => {
          if (this.memory.Xtream) {
            this.chkSerie = !this.chkSerie;
            this.load();
          }
        }
      },
      {
        key: "left",
        label: "Navigation",
        description: "Go left",
        allowIn: [AllowIn.Input],
        command: _ => this.nav("ArrowLeft")
      },
      {
        key: "shift + tab",
        label: "Navigation",
        description: "Go left",
        allowIn: [AllowIn.Input],
        preventDefault: true,
        command: _ => this.nav("ShiftTab")
      },
      {
        key: "right",
        label: "Navigation",
        description: "Go right",
        allowIn: [AllowIn.Input],
        command: _ => this.nav("ArrowRight")
      },
      {
        key: "tab",
        label: "Navigation",
        description: "Go right",
        allowIn: [AllowIn.Input],
        preventDefault: true,
        command: _ => this.nav("Tab")
      },
      {
        key: "up",
        label: "Navigation",
        description: "Go up",
        allowIn: [AllowIn.Input],
        preventDefault: true,
        command: _ => this.nav("ArrowUp")
      },
      {
        key: "down",
        label: "Navigation",
        description: "Go down",
        allowIn: [AllowIn.Input],
        preventDefault: true,
        command: _ => this.nav("ArrowDown")
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

  filtersVisible() {
    return (this.viewMode != this.viewModeEnum.Categories || this.memory.SelectedCategory) && !this.memory.SelectedSerie
  }

  switchMode(viewMode: ViewMode) {
    if (viewMode == this.viewMode)
      return;
    this.elementsToRetrieve = this.defaultElementsToRetrieve;
    this.memory.clearCategoryNode();
    this.memory.clearSeriesNode();
    this.viewMode = viewMode;
    this.search.nativeElement.value = "";
    this.lastTerm = "";
    this.load();
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

  getChannels() {
    this.channels = this.memory.Channels.slice(0, this.elementsToRetrieve);
    this.channelsLeft = this.memory.Channels.length - this.elementsToRetrieve;
    this.favChannels = this.memory.FavChannels;
  }

  getCategories() {
    let tmpDic: any = {}
    this.memory.Channels.forEach(x => {
      if (x.group?.trim() && !tmpDic[x.group]) {
        let group: Channel = {
          name: x.group,
          group: x.group,
          image: x.image,
          url: "",
          type: MediaType.group
        }
        tmpDic[x.group] = group;
      }
    });

    this.memory.Categories = Object.values(tmpDic);
    this.categories = this.memory.Categories.slice(0, this.elementsToRetrieve);
  }

  filterChannels(term: string) {
    let params = this.getFilteringParameters();
    let allowedTypes = params.useFilters ? this.getAllowedMediaTypes() : null
    let result = params.source
      .filter(y => y.name.toLowerCase().indexOf(term.toLowerCase()) > -1
        && (params.useFilters ? allowedTypes?.includes(y.type) : true))
    this.channelsLeft = result.length - this.elementsToRetrieve;
    result = result.slice(0, this.elementsToRetrieve);
    return result;
  }

  getAllowedMediaTypes(): Array<MediaType> {
    let array = [];
    if (this.chkLivestream)
      array.push(MediaType.livestream);
    if (this.chkMovie)
      array.push(MediaType.movie);
    if (this.chkSerie)
      array.push(MediaType.serie);
    return array;
  }

  getFilteringParameters() {
    if (this.memory.SelectedSerie)
      return { source: this.memory.SeriesNode, useFilters: false };
    switch (this.viewMode) {
      case this.viewModeEnum.All:
        return { source: this.memory.Channels, useFilters: true };
      case this.viewModeEnum.Favorites:
        return { source: this.memory.FavChannels, useFilters: true };
      case this.viewModeEnum.Categories:
        if (this.memory.SelectedCategory)
          return { source: this.memory.CategoriesNode, useFilters: true }
        return { source: this.memory.Categories, useFilters: false };
    }
  }

  goBackHotkey() {
    if (this.memory.CategoriesNode) {
      if (this.focusArea == FocusArea.Filters) {
        this.focusArea = FocusArea.Tiles;
        this.focus = 0;
      }
      this.goBack();
    }
    this.closeContextMenu();
    this.selectFirstChannel();
  }

  goBack() {
    if(this.memory.SelectedSerie)
      this.memory.clearSeriesNode();
    else {
      this.memory.clearCategoryNode();
    }
    this.load();
  }

  openSettings() {
    this.router.navigateByUrl("settings");
  }

  nav(key: string) {
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
    else if (this.focusArea == FocusArea.Tiles && tmpFocus >= this.elementsToRetrieve && this.channelsLeft > 0)
      this.loadMore();
    else {
      if (tmpFocus >= this.channels.length && this.focusArea == FocusArea.Tiles)
        tmpFocus = (this.channels.length == 0 ? 1 : this.channels.length) - 1;
      this.focus = tmpFocus;
      document.getElementById(`${FocusAreaPrefix[this.focusArea]}${this.focus}`)?.focus();
    }
  }

  shortFiltersMode() {
    return !this.memory.Xtream && this.focusArea == FocusArea.Filters;
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
