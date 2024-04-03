import { Component, Input, OnInit } from '@angular/core';
import { NgbActiveModal } from '@ng-bootstrap/ng-bootstrap';
import { ToastrService } from 'ngx-toastr';
import { SetupMode } from '../models/setupMode';
import { Xtream } from '../models/xtream';
import { Source } from '../models/source';
import { MemoryService } from '../memory.service';

@Component({
  selector: 'app-edit-modal',
  templateUrl: './edit-modal.component.html',
  styleUrls: ['./edit-modal.component.scss'],
})
export class EditModalComponent implements OnInit {
  constructor(
    public memory: MemoryService,
    public activeModal: NgbActiveModal,
    private toastr: ToastrService
  ) {}
  @Input() source!: Source;
  loading = false;
  electron: any = (window as any).electronAPI;
  setupModeEnum = SetupMode;
  setupMode: SetupMode = SetupMode.m3uFile;
  name?: string;
  url?: string;
  xtream: Xtream = {};

  ngOnInit(): void {
    const { name, url, xtream } = this.source;
    this.name = name;

    if (url) {
      this.setupMode = this.setupModeEnum.m3u;
      this.url = url;
    }

    if (xtream) {
      this.setupMode = this.setupModeEnum.xtream;
      this.xtream = xtream;
    }
  }

  async submit() {
    const result = await this.electron.editSource(
      this.memory.Name,
      this.name,
      this.url,
      this.xtream
    );
    if (result) {
      this.memory.clearAllSource();
      window.location.reload();
      this.activeModal.close();
    } else this.error();
  }

  error() {
    this.toastr.error(
      'Invalid URL or credentials. Try again with the same or a different URL'
    );
  }
}
