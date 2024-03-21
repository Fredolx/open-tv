import { Component } from '@angular/core';
import { Router } from '@angular/router';
import { ToastrService } from 'ngx-toastr';
import { NgbModal } from '@ng-bootstrap/ng-bootstrap';
import { Cache } from '../models/cache';
import { MemoryService } from '../memory.service';
import { Favs, Source } from '../models/source';
import { DeleteModalComponent } from '../delete-modal/delete-modal.component';

@Component({
  selector: 'app-sources',
  templateUrl: './sources.component.html',
  styleUrls: ['./sources.component.scss'],
})
export class SourcesComponent {
  sources: Source[] = [];
  electron: any = (window as any).electronAPI;
  edit: Boolean = false;
  loading: Boolean = false;

  constructor(
    private router: Router,
    public memory: MemoryService,
    public toast: ToastrService,
    private modalService: NgbModal
  ) {
    this.loading = true;
    if (this.memory.Sources.length > 0) {
      this.sources = this.memory.Sources;
      this.loading = false;
    } else
      this.electron
        .getCache()
        .then((x: { cache: [Cache]; favs: Favs[] }) => {
          if (x.cache?.length > 0) {
            const newCaches = x.cache.map((ca) => ({
              ...ca,
              favs: x.favs.find((e) => e.name === ca.name)?.channels || [],
            }));
            this.sources = newCaches;
            this.memory.Sources = this.sources;
          } else this.goToSetup();
          this.loading = false;
        })
        .catch(() => {
          this.loading = false;
          this.toast.error('Could not load cached settings');
          this.reset();
        });
  }

  reset() {
    this.electron.deleteAllCache().finally(() => {
      this.router.navigateByUrl('setup');
    });
  }

  goToSetup() {
    this.router.navigateByUrl('setup');
  }

  editSources() {
    if (this.edit) return (this.edit = false);

    return (this.edit = true);
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

  async deleteModal(source: Source) {
    let result = await this.modalService.open(DeleteModalComponent, {
      keyboard: false,
      backdrop: 'static',
      size: 'lg',
    }).result;

    if (result === 'delete') {
      const { name } = source;

      const newSources = this.memory.Sources.filter(
        (source) => source.name !== name
      );

      this.sources = newSources;
      this.memory.Sources = this.sources;
      this.electron.deleteCache(name);

      if (this.sources.length === 0) {
        this.edit = false;
        this.goToSetup();
      }
    }
  }
}
