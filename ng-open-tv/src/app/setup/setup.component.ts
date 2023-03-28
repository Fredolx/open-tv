import { Component, OnInit } from '@angular/core';
import { MemoryService } from '../memory.service';
import { Channel } from '../models/channel';
import { Router } from '@angular/router';
import { ToastrService } from 'ngx-toastr';
import { SetupMode } from '../models/setupMode';
import { Xtream } from '../models/xtream';

@Component({
  selector: 'app-setup',
  templateUrl: './setup.component.html',
  styleUrls: ['./setup.component.scss']
})
export class SetupComponent {
  constructor(public memory: MemoryService, private nav: Router, private toastr: ToastrService) { }
  url?: string
  loading = false;
  electron: any = (window as any).electronAPI;
  setupModeEnum = SetupMode;
  setupMode: SetupMode = SetupMode.m3uFile;
  xtream: Xtream = {}

  ngOnInit(): void {
  }

  async getFile() {
    this.loading = true;
    let result = await this.electron.selectFile();
    if (result) {
      this.memory.Channels = result;
      this.nav.navigateByUrl("");
    }
    else
      this.toastr.error("Could not parse selected file");
    this.loading = false;
  }

  async getFileFromURL() {
    this.loading = true;
    if (await this.memory.DownloadM3U(this.url))
      this.nav.navigateByUrl("");
    else
      this.toastr.error("Invalid URL or credentials. Try again with the same or a different URL");
    this.loading = false;
  }

  async submitXtream() {
    this.loading = true;
    if (await this.memory.GetXtream(this.xtream))
      this.nav.navigateByUrl("");
    else
      this.toastr.error("Invalid URL or credentials. Try again with the same or a different URL");
    this.loading = false;
  } 
}
