import { Injectable } from '@angular/core';
import { Subject } from 'rxjs';
import { Channel } from './models/channel';
import { Settings } from './models/settings';
import { Xtream } from './models/xtream';
import { MatMenuTrigger } from '@angular/material/menu';
import { MediaType } from './models/mediaType';

@Injectable({
  providedIn: 'root'
})
export class MemoryService {
  constructor() { }
  public Channels: Channel[] = [];
  public FavChannels: Channel[] = [];
  public Categories: Channel[] = [];
  public CategoriesNode: Channel[] = [];
  public SeriesNode: Channel[] = [];
  public SelectedSerie?: Channel;
  public SelectedCategory?: Channel;
  public StartingChannel: boolean = false;
  public NeedToRefreshFavorites: Subject<boolean> = new Subject();
  public SwitchingNode: Subject<boolean> = new Subject();
  public Url?: String
  public Settings: Settings = {};
  private electron: any = (window as any).electronAPI;
  public Xtream?: Xtream;
  public currentContextMenu?: MatMenuTrigger
  public Loading = false;

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
    if (channels?.length > 0) {
      this.Channels = channels;
      return true;
    }
    return false;
  }

  async GetXtream(xtream: Xtream | undefined = undefined) {
    let channels;
    if (xtream)
      this.Xtream = xtream;
    try {
      channels = await this.electron.getXtream(this.Xtream);
    }
    catch (e) {
      console.error(e);
      return false;
    }
    if (channels?.length > 0) {
      this.Channels = channels;
      return true;
    }
    return false;
  }

  clearSeriesNode() {
    this.SelectedSerie = undefined;
    this.SeriesNode = [];
  }

  clearCategoryNode() {
    this.SelectedCategory = undefined;
    this.CategoriesNode = [];
  }

  async setNode(channel: Channel) {
    if (channel.type == MediaType.serie) {
      this.SelectedSerie = channel;
      this.Loading = true;
      this.SeriesNode = await this.electron.getEpisodes({ seriesId: channel.url, xtream: this.Xtream });
      this.Loading = false;
    }
    else {
      this.SelectedCategory = channel;
      this.CategoriesNode = this.Channels.filter(x => x.group === channel.group);
    }
    this.SwitchingNode.next(true);
  }
}
