import { Component, OnInit } from '@angular/core';
import { Router } from '@angular/router';
import { NgbActiveModal, NgbModalRef } from '@ng-bootstrap/ng-bootstrap';
import { invoke } from '@tauri-apps/api';

@Component({
  selector: 'app-settings',
  templateUrl: './settings.component.html',
  styleUrls: ['./settings.component.css']
})
export class SettingsComponent implements OnInit {

  loading = false;
  constructor(private router: Router) { }

  ngOnInit(): void {
  }

  async deleteCache(){
    this.loading = true;
    await invoke("delete_cache");
    this.loading = false;
    this.router.navigateByUrl("setup");
  }

  goBack(){
    this.router.navigateByUrl("");
  }

}
