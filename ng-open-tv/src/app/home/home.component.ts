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
  lastTermFavs?: string;
  @ViewChild('search') search!: ElementRef;
  @ViewChild('searchFavs') searchFavs!: ElementRef;
  defaultElementsToRetrieve: number = 36;
  elementsToRetrieve: number = this.defaultElementsToRetrieve;
  channelsLeft: number = 0;
  shortcuts: ShortcutInput[] = [];
  chkLivestream: boolean = true;
  chkMovie: boolean = true;
  chkSerie: boolean = true;

  constructor(private router: Router, public memory: MemoryService, public toast: ToastrService) {
    if (this.memory.Channels.length > 0) {
      this.getChannels();
    }
    else {
      this.electron.getCache().then((x: { cache: Cache, favs: Channel[], performedMigration?: boolean }) => {
        if (x.cache?.channels?.length > 0) {
          if (x.performedMigration)
            toast.info("Your channel data has been migrated. Please delete & re-load your channels if you notice any issues", undefined, {timeOut: 20000})
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
          this.memory.NeedToRefreshFavorites.subscribe(_ => {
            if (this.lastTermFavs?.trim() && this.favChannels.length > 1)
              this.favChannels = this.filterFavs(this.lastTermFavs);
            else {
              if (this.lastTermFavs?.trim())
                this.searchFavs.nativeElement.value = "";
              this.favChannels = this.memory.FavChannels;
            }
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
    if (this.getAllowedMediaTypes().length == 3 && !this.lastTerm)
      this.channels = this.memory.Channels.slice(0, this.elementsToRetrieve);
    else
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

    fromEvent(this.searchFavs.nativeElement, 'keyup').pipe(
      map((event: any) => {
        return event.target.value;
      })
      , debounceTime(300)
      , distinctUntilChanged()
    ).subscribe((term: string) => {
      this.lastTermFavs = term;
      this.favChannels = this.filterFavs(term);
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

  focusSearch() {
    let element = this.viewMode == this.viewModeEnum.All ?
      this.search.nativeElement : this.searchFavs.nativeElement;
    window.scrollTo({ top: 0, behavior: 'smooth' });
    element.focus({
      preventScroll: true
    });
  }

  getChannels() {
    this.channels = this.memory.Channels.slice(0, this.elementsToRetrieve);
    this.channelsLeft = this.memory.Channels.length - this.elementsToRetrieve;
    this.favChannels = this.memory.FavChannels;
  }

  filterChannels(term: string) {
    let allowedTypes = this.getAllowedMediaTypes();
    let result = this.memory.Channels
      .filter(y => y.name.toLowerCase().indexOf(term.toLowerCase()) > -1 && allowedTypes.includes(y.type))
    this.channelsLeft = result.length - this.elementsToRetrieve;
    result = result.slice(0, this.elementsToRetrieve);
    return result;
  }

  filterFavs(term: string) {
    let result = this.memory.FavChannels
      .filter(y => y.name.toLowerCase().indexOf(term.toLowerCase()) > -1)
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

  openSettings() {
    this.router.navigateByUrl("settings");
  }
}
