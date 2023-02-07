import { Injectable } from '@angular/core';
import { Subject } from 'rxjs';
import { Channel } from './models/channel';

@Injectable({
  providedIn: 'root'
})
export class MemoryService {
  constructor() { }
  public Channels: Channel[] = [];
  public FavChannels: Channel[] = [];
  public startingChannel: boolean = false;
  public needToRefreshFavorites: Subject<boolean> = new Subject();
}
