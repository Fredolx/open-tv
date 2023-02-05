import { Component, OnInit } from '@angular/core';
import { Router } from '@angular/router';
import { NgbActiveModal, NgbModalRef } from '@ng-bootstrap/ng-bootstrap';
import { MemoryService } from '../memory.service';

@Component({
  selector: 'app-settings',
  templateUrl: './settings.component.html',
  styleUrls: ['./settings.component.scss']
})
export class SettingsComponent {
  loading = false;
  electron: any = (window as any).electronAPI;

  constructor(private router: Router, private memory: MemoryService) { }

  ngOnInit(): void {
  }

  async deleteCache(){
    this.loading = true;
    await this.electron.deleteCache();
    this.memory.Channels = [];
    this.memory.FavChannels = [];
    this.loading = false;
    this.router.navigateByUrl("setup");
  }

  goBack(){
    this.router.navigateByUrl("");
  }

}
