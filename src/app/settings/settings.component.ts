import { Component, ElementRef, ViewChild } from '@angular/core';
import { debounceTime, distinctUntilChanged, fromEvent, map, Subscription } from 'rxjs';
import { Settings } from '../models/settings';
import { invoke } from '@tauri-apps/api/core';
import { Router } from '@angular/router';
import { ToastrService } from 'ngx-toastr';
import { open } from '@tauri-apps/plugin-dialog';
import { Source } from '../models/source';

@Component({
  selector: 'app-settings',
  templateUrl: './settings.component.html',
  styleUrl: './settings.component.css'
})
export class SettingsComponent {
  loading = false;
  subscriptions: Subscription[] = [];
  settings: Settings = {
    use_stream_caching: true,
  };
  sources: Source[] = [];
  @ViewChild('mpvParams') mpvParams!: ElementRef;

  constructor(private router: Router, private toastr: ToastrService) { }

  ngOnInit(): void {
    invoke('get_settings').then(x => this.settings = x as Settings);
    invoke('get_sources').then(x => this.sources = x as Source[]);
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
  }

  async deleteSource(id: number) {
    this.tryIPC("Successfully deleted source", "Failed to delete source", () => invoke("delete_source", {id: id}));
    //Reload sources?
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

  async tryIPC<T>(
    successMessage: string,
    errorMessage: string,
    action: () => Promise<T>
  ): Promise<void> {
    this.loading = true;
    try {
      await action();
      this.toastr.success(successMessage);
    } catch (e) {
      this.toastr.error(errorMessage);
    }
    this.loading = false;
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

  ngOnDestroy(): void {
    this.subscriptions.forEach(x => x.unsubscribe());
  }
}
