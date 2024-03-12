import { Component } from '@angular/core';
import { Cache } from '../models/cache';
import { Router } from '@angular/router';
import { MemoryService } from '../memory.service';
import { ToastrService } from 'ngx-toastr';
import { Channel } from '../models/channel';
import { Source } from '../models/source';

@Component({
  selector: 'app-sources',
  templateUrl: './sources.component.html',
  styleUrls: ['./sources.component.scss'],
})
export class SourcesComponent {
  sources: Source[] = [];
  electron: any = (window as any).electronAPI;
  loading: Boolean = false;

  constructor(
    private router: Router,
    public memory: MemoryService,
    public toast: ToastrService
  ) {
    this.loading = true;
    this.electron
      .getCache()
      .then((x: { cache: [Cache]; favs: Channel[] }) => {
        if (x.cache?.length > 0) {
          const newCaches = x.cache.map((ca) => ({
            ...ca,
            favs: x.favs,
          }));
          this.sources = newCaches;
          this.memory.Sources = this.sources;

          if (this.sources.length === 1) this.click(this.sources[0])
        } else this.reset();
        this.loading = false;
      })
      .catch(() => {
        this.loading = false;
        this.toast.error('Could not load cached settings');
        this.reset();
      });
  }

  reset() {
    this.electron.deleteCache().finally(() => {
      this.router.navigateByUrl('setup');
    });
  }

  async click(source: Source) {
    const { name, channels, xtream, url, favs } = source;
    if (channels?.length > 0) this.memory.Channels = channels;
    if (name) this.memory.Name = name;
    if (xtream) this.memory.Xtream = xtream;
    if (url) this.memory.Url = url;
    if (favs) this.memory.FavChannels = favs;

    this.router.navigateByUrl('channels');
  }
}
