import { Injectable } from '@angular/core';
import { Subject } from 'rxjs';
import { Channel } from './models/channel';
import { Settings } from './models/settings';

@Injectable({
  providedIn: 'root'
})
export class MemoryService {
  constructor() { }
  public Channels: Channel[] = [];
  public FavChannels: Channel[] = [];
  public StartingChannel: boolean = false;
  public NeedToRefreshFavorites: Subject<boolean> = new Subject();
  public Url?: String
  public Settings: Settings = {};
  private electron: any = (window as any).electronAPI;

  async DownloadM3U(url: String | undefined = undefined): Promise<boolean> {
    let channels;
    if (url?.trim())
      this.Url = url.trim();
    try {
      channels = await this.electron.downloadM3U(this.Url);
    }
    catch (e) {
      console.error(e);
      return false;
    }
    if (channels && channels.length > 0) {
      this.Channels = channels;
      return true;
    }
    this.Url = undefined;
    return false;
  }
}
