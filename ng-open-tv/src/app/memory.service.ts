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
  public Name: String = '';
  public Url?: String;
  public Settings: Settings = {};
  private electron: any = (window as any).electronAPI;
  public Xtream?: Xtream;
  public currentContextMenu?: MatMenuTrigger
  public Loading = false;

  async GetFile(name: String | undefined = undefined): Promise<boolean> {
    let channels;
    if (name?.trim())
      this.Name = name.trim();
    try {
      channels = await this.electron.selectFile(this.Name);
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

  async DownloadM3U(name: String | undefined = undefined, url: String | undefined = undefined): Promise<boolean> {
    let channels;
    if (name?.trim())
      this.Name = name.trim();
    if (url?.trim())
      this.Url = url.trim();
    try {
      channels = await this.electron.downloadM3U(this.Name, this.Url);
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

  async GetXtream(name: String | undefined = undefined, xtream: Xtream | undefined = undefined) {
    let channels;
    if (name)
      this.Name = name;
    if (xtream)
      this.Xtream = xtream;
    try {
      channels = await this.electron.getXtream(this.Name, this.Xtream);
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
