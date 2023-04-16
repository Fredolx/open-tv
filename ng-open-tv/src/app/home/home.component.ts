import { AfterViewInit, Component, ElementRef, HostListener, ViewChild } from '@angular/core';
import { Router } from '@angular/router';
import { AllowIn, ShortcutInput } from 'ng-keyboard-shortcuts';
import { debounceTime, distinctUntilChanged, fromEvent, map } from 'rxjs';
import { MemoryService } from '../memory.service';
import { Cache } from '../models/cache';
import { Channel } from '../models/channel';
import { ViewMode } from '../models/viewMode';
import { MediaType } from '../models/mediaType';
import { ToastrService } from 'ngx-toastr';

@Component({
  selector: 'app-home',
  templateUrl: './home.component.html',
  styleUrls: ['./home.component.scss']
})
export class HomeComponent implements AfterViewInit {
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

  constructor(private router: Router, public memory: MemoryService, public toast: ToastrService) {
    if (this.memory.Channels.length > 0) {
      this.getChannels();
      this.getCategories();
    }
    else {
      this.electron.getCache().then((x: { cache: Cache, favs: Channel[], performedMigration?: boolean }) => {
        if (x.cache?.channels?.length > 0) {
          if (x.performedMigration)
            toast.info("Your channel data has been migrated. Please delete & re-load your channels if you notice any issues", undefined, { timeOut: 20000 })
          this.memory.Channels = x.cache.channels;
          if (x.cache.username?.trim())
            this.memory.Xtream =
            {
              url: x.cache.url,
              username: x.cache.username,
              password: x.cache.password
            }
          else
            this.memory.Url = x.cache.url;
          this.memory.FavChannels = x.favs;
          this.getChannels();
          this.getCategories();
          this.memory.NeedToRefreshFavorites.subscribe(_ => {
            this.load();
          });
          this.memory.SwitchToCategoriesNode.subscribe(_ => {
            this.load();
          });
        }
        else
          router.navigateByUrl("setup");
      });
    }
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

  ngAfterViewInit(): void {
    fromEvent(this.search.nativeElement, 'keyup').pipe(
      map((event: any) => {
        return event.target.value;
      })
      , debounceTime(300)
      , distinctUntilChanged()
    ).subscribe((term: string) => {
      this.elementsToRetrieve = this.defaultElementsToRetrieve;
      this.lastTerm = term;
      this.channels = this.filterChannels(term);
    });

    this.shortcuts.push(
      {
        key: "ctrl + f",
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
        allowIn: [AllowIn.Input],
        command: _ => this.viewMode = this.viewModeEnum.All
      },
      {
        key: "ctrl + s",
        label: "Switching modes",
        description: "Selects the favorites channels mode",
        allowIn: [AllowIn.Input],
        command: _ => this.viewMode = this.viewModeEnum.Favorites
      },
      {
        key: "ctrl + d",
        label: "Quick navigation",
        description: "Selects the first channel. Press tab/shift+tab for next/previous",
        allowIn: [AllowIn.Input],
        command: _ => (document.getElementById('first')?.firstChild as HTMLElement)?.focus()
      },
      {
        key: "ctrl + q",
        label: "Media Type Filters",
        description: "Enable/Disable livestreams",
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
        allowIn: [AllowIn.Input],
        command: _ => {
          if (this.memory.Xtream) {
            this.chkSerie = !this.chkSerie;
            this.load();
          }
        }
      },
    );
  }

  switchMode(viewMode: ViewMode) {
    if(viewMode == this.viewMode)
      return;
    this.elementsToRetrieve = this.defaultElementsToRetrieve;
    this.memory.clearCategoryNode();
    this.viewMode = viewMode;
    this.search.nativeElement.value = "";
    this.lastTerm = "";
    this.load();
  }

  focusSearch() {
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
      if (x.group.trim() && !tmpDic[x.group] && x.type == MediaType.livestream) {
        x.name = x.group;
        x.type = MediaType.group;
        tmpDic[x.group] = x;
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

  goBack() {
    this.memory.clearCategoryNode();
    this.load();
  }

  openSettings() {
    this.router.navigateByUrl("settings");
  }
}
