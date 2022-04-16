import { Component, OnInit } from '@angular/core';
import { invoke } from '@tauri-apps/api/tauri'
import { open, save } from "@tauri-apps/api/dialog"
import { MemoryService } from '../memory.service';
import { Channel } from '../models/channel';
import { Router } from '@angular/router';
import { ToastrService } from 'ngx-toastr';

@Component({
  selector: 'app-setup',
  templateUrl: './setup.component.html',
  styleUrls: ['./setup.component.css']
})
export class SetupComponent implements OnInit {

  constructor(public memory: MemoryService, private nav: Router, private toastr: ToastrService) { }
  url?: string
  loading = false;
  ngOnInit(): void {
  }

  async getFile() {
    this.url = (await open() as string);
    if (!this.url)
      return;
    this.loading = true;
    let result = await invoke("get_playlist", { url: this.url });
    if (result){
      this.nav.navigateByUrl("");
    }
    else
      this.toastr.error("Could not parse selected file");
    this.loading = false;
  }

}
