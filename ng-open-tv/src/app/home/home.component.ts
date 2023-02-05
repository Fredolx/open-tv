import { AfterViewInit, Component, ElementRef, OnInit, ViewChild } from '@angular/core';
import { Router } from '@angular/router';
import { NgbModal } from '@ng-bootstrap/ng-bootstrap';
import { debounceTime, distinctUntilChanged, filter, fromEvent, map, Subject, tap } from 'rxjs';
import { MemoryService } from '../memory.service';
import { Channel } from '../models/channel';
import { ViewMode } from '../models/viewMode';

@Component({
  selector: 'app-home',
  templateUrl: './home.component.html',
  styleUrls: ['./home.component.scss']
})
export class HomeComponent {
  channels: Channel[] = [];
  favChannels: Channel[] = [];
  viewMode = ViewMode.All;
  ViewModeEnum = ViewMode;
  electron: any = (window as any).electronAPI;
  @ViewChild('search') search!: ElementRef;
  @ViewChild('searchFavs') searchFavs!: ElementRef;
  readonly elementsToRetrieve = 36;

  constructor(private router: Router, public memory: MemoryService) {
    if (this.memory.Channels.length > 0){
      this.getChannels();
    }
    else {
      this.electron.getCache().then((x: any) => {
        if (x) {
          this.memory.Channels = x as Channel[];
          this.getChannels();
        }
        if (memory.Channels?.length == 0)
          router.navigateByUrl("setup");
      });
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
      this.filterChannels(term, this.memory.Channels, this.channels);
    });

    fromEvent(this.searchFavs.nativeElement, 'keyup').pipe(
      map((event: any) => {
        return event.target.value;
      })
      , debounceTime(300)
      , distinctUntilChanged()
    ).subscribe((term: string) => {
      this.filterChannels(term, this.memory.FavChannels, this.favChannels);
    });
  }

  getChannels(){
    this.channels = this.memory.Channels.slice(0, this.elementsToRetrieve);
  }

  filterChannels(term: string, source: Channel[], target: Channel[]){
    target = source
      .filter(y => y.name.toLowerCase().indexOf(term.toLowerCase()) > -1)
      .slice(0, this.elementsToRetrieve)
  }

  openSettings(){
    this.router.navigateByUrl("settings");
  }
}
