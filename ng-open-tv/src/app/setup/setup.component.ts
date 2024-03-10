import { Component, OnInit } from '@angular/core';
import { MemoryService } from '../memory.service';
import { Channel } from '../models/channel';
import { Router } from '@angular/router';
import { ToastrService } from 'ngx-toastr';
import { SetupMode } from '../models/setupMode';
import { Xtream } from '../models/xtream';
import { NgbModal } from '@ng-bootstrap/ng-bootstrap';
import { ConfirmModalComponent } from '../confirm-modal/confirm-modal.component';

@Component({
  selector: 'app-setup',
  templateUrl: './setup.component.html',
  styleUrls: ['./setup.component.scss']
})
export class SetupComponent {
  constructor(public memory: MemoryService, private nav: Router,
    private toastr: ToastrService, private modalService: NgbModal) { }
  name?: string;
  url?: string;
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
      this.error();
    this.loading = false;
  }

  async submitXtream() {
    this.loading = true;
    this.name = this.name?.trim();
    this.xtream.url = this.xtream.url?.trim();
    this.xtream.username = this.xtream.username?.trim();
    this.xtream.password = this.xtream.password?.trim();
    if (!this.xtream.url || !this.xtream.username || !this.xtream.password) {
      this.error();
      this.loading = false;
      return;
    }
    if(!this.xtream.url.startsWith('http://') && !this.xtream.url.startsWith('https://')){
      this.xtream.url = `http://${this.xtream.url}`;
      this.toastr.info("Since the given URL lacked a protocol, http was assumed");
    }
    let url = new URL(this.xtream.url);
    if (url.pathname == "/") {
      let result = await this.modalService.open
        (ConfirmModalComponent, { keyboard: false, backdrop: 'static' }).result;
      if (result == "correct") {
        url.pathname = "/player_api.php";
        this.xtream.url = url.toString();
      }
    }
    if (await this.memory.GetXtream(this.name, this.xtream))
      this.nav.navigateByUrl("");
    else
      this.error();
    this.loading = false;
  }
  error() {
    this.toastr.error("Invalid URL or credentials. Try again with the same or a different URL");
  }
}
