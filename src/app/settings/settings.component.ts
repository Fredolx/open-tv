import { Component, ElementRef, ViewChild } from '@angular/core';
import { debounceTime, distinctUntilChanged, fromEvent, map, Subscription } from 'rxjs';
import { Settings } from '../models/settings';
import { invoke } from '@tauri-apps/api/core';
import { Router } from '@angular/router';
import { open } from '@tauri-apps/plugin-dialog';
import { Source } from '../models/source';
import { MemoryService } from '../memory.service';
import { ViewMode } from '../models/viewMode';
import { EditChannelModalComponent } from '../edit-channel-modal/edit-channel-modal.component';
import { NgbModal } from '@ng-bootstrap/ng-bootstrap';
import { ImportModalComponent } from '../import-modal/import-modal.component';

@Component({
  selector: 'app-settings',
  templateUrl: './settings.component.html',
  styleUrl: './settings.component.css'
})
export class SettingsComponent {
  subscriptions: Subscription[] = [];
  settings: Settings = {
    use_stream_caching: true,
    default_view: ViewMode.All,
    volume: 100
  };
  viewModeEnum = ViewMode;
  sources: Source[] = [];
  @ViewChild('mpvParams') mpvParams!: ElementRef;

  constructor(private router: Router, public memory: MemoryService, private nav: Router, private modal: NgbModal) { }

  ngOnInit(): void {
    this.getSettings();
    this.getSources();
  }

  getSettings() {
    invoke('get_settings').then(x => {
      this.settings = x as Settings
      if (this.settings.use_stream_caching == undefined)
        this.settings.use_stream_caching = true;
      if (this.settings.default_view == undefined)
        this.settings.default_view = ViewMode.All;
      if (this.settings.volume == undefined)
        this.settings.volume = 100;
    });
  }

  getSources() {
    invoke('get_sources').then(x => {
      this.sources = x as Source[];
      if (this.sources.length == 0) {
        this.memory.AddingAdditionalSource = false;
        this.nav.navigateByUrl("setup");
      }
    });
  }

  ngAfterViewInit(): void {
    this.subscriptions.push(
      fromEvent(this.mpvParams.nativeElement, 'keyup').pipe(
        map((event: any) => {
          return event.target.value;
        })
        , debounceTime(500)
        , distinctUntilChanged()
      ).subscribe(async () => {
        await this.updateSettings();
      }));
    this.subscriptions.push(this.memory.RefreshSources.subscribe(_ => {
      this.getSources();
    }));
  }

  addSource() {
    this.memory.AddingAdditionalSource = true;
    this.nav.navigateByUrl("setup");
  }

  async refreshAll() {
    await this.memory.tryIPC("Successfully updated all sources", "Failed to refresh sources", () => invoke("refresh_all"));
  }

  async goBack() {
    await this.updateSettings();
    this.router.navigateByUrl("");
  }

  async updateSettings() {
    if (this.settings.mpv_params)
      this.settings.mpv_params = this.settings.mpv_params?.trim();
    await invoke("update_settings", {settings: this.settings});
  }

  async selectFolder() {
    const folder = await open({
      multiple: false,
      directory: true,
      canCreateDirectories: true,
    });
    if (folder) {
      this.settings.recording_path = folder;
      await this.updateSettings();
    }
  }

  import() {
    const modalRef = this.modal.open(ImportModalComponent, { backdrop: 'static', size: 'xl', });
    modalRef.componentInstance.name = "ImportModalComponent";
    modalRef.componentInstance.onlyPlaylists = true;
  }

  async deleteAll() {
    this.memory.tryIPC("Successfully deleted everything", "Failed to delete everything", () => invoke('delete_database'))
  }

  ngOnDestroy(): void {
    this.subscriptions.forEach(x => x.unsubscribe());
  }
}
