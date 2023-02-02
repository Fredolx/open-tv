import { AfterViewInit, Component, ElementRef, OnInit, ViewChild } from '@angular/core';
import { Router } from '@angular/router';
import { NgbModal } from '@ng-bootstrap/ng-bootstrap';
import { debounceTime, distinctUntilChanged, filter, fromEvent, map, Subject, tap } from 'rxjs';
import { MemoryService } from '../memory.service';
import { Channel } from '../models/channel';

@Component({
  selector: 'app-home',
  templateUrl: './home.component.html',
  styleUrls: ['./home.component.scss']
})
export class HomeComponent {
  channels: Channel[] = [];
  electron: any = (window as any).electronAPI;
  @ViewChild('search') search!: ElementRef;
  readonly elementsToRetrieve = 36;

  constructor(private router: Router, public memory: MemoryService) {
    console.dir(this.electron);
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
      this.filterChannels(term);
    });
  }

  getChannels(){
    this.channels = this.memory.Channels.slice(0, this.elementsToRetrieve);
  }

  filterChannels(term: string){
    this.channels = this.memory.Channels
      .filter(y => y.name.toLowerCase().indexOf(term.toLowerCase()) > -1)
      .slice(0, this.elementsToRetrieve)
  }

  openSettings(){
    this.router.navigateByUrl("settings");
  }
}
