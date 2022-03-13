import { Injectable } from '@angular/core';
import { Channel } from './models/channel';

@Injectable({
  providedIn: 'root'
})
export class MemoryService {

  constructor() { }
  public Channels: Channel[] = [];
}
