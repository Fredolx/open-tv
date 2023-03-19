import { Component, OnInit } from '@angular/core';
import { Router } from '@angular/router';
import { NgbActiveModal, NgbModalRef } from '@ng-bootstrap/ng-bootstrap';
import { ToastrService } from 'ngx-toastr';
import { MemoryService } from '../memory.service';
import { Settings } from '../models/settings';

@Component({
  selector: 'app-settings',
  templateUrl: './settings.component.html',
  styleUrls: ['./settings.component.scss']
})
export class SettingsComponent {
  loading = false;
  electron: any = (window as any).electronAPI;

  constructor(private router: Router, public memory: MemoryService, private toastr: ToastrService) { }

  ngOnInit(): void {
    this.electron.getSettings().then((x: Settings) => {
      if (x)
        this.memory.Settings = x;
    });
  }

  async refresh() {
    this.loading = true;
    if (await this.memory.DownloadM3U())
      this.toastr.success("Your channels are now up-to-date");
    else
      this.toastr.error("Error while trying to refresh channels, try again or delete channels cache");
    this.loading = false;
  }

  async deleteCache() {
    if (this.loading == true)
      return;
    this.loading = true;
    await this.electron.deleteCache();
    this.memory.Channels = [];
    this.memory.FavChannels = [];
    this.loading = false;
    this.router.navigateByUrl("setup");
  }

  async selectFolder() {
    let result = await this.electron.selectFolder();
    if (result)
      this.memory.Settings.recordingPath = result;
    await this.updateSettings();
  }

  goBack() {
    this.router.navigateByUrl("");
  }

  async updateSettings() {
    await this.electron.updateSettings(this.memory.Settings);
  }

  CanRefresh(): boolean {
    return this.memory.Url?.trim() ? true : false;
  }
}
