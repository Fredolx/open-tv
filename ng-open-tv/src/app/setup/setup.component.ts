import { Component, OnInit } from '@angular/core';
import { MemoryService } from '../memory.service';
import { Channel } from '../models/channel';
import { Router } from '@angular/router';
import { ToastrService } from 'ngx-toastr';

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
  ngOnInit(): void {
  }

  async getFile() {
    this.loading = true;
    let result = await this.electron.selectFile();
    console.dir(result);
    if (result){
      this.memory.Channels = result;
      this.nav.navigateByUrl("");
    }
    else
      this.toastr.error("Could not parse selected file");
    this.loading = false;
  }
} 
