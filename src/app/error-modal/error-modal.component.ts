import { Component } from '@angular/core';
import { NgbActiveModal } from '@ng-bootstrap/ng-bootstrap';
import { writeText } from '@tauri-apps/plugin-clipboard-manager';
import { ToastrService } from 'ngx-toastr';

@Component({
  selector: 'app-error-modal',
  templateUrl: './error-modal.component.html',
  styleUrl: './error-modal.component.css'
})
export class ErrorModalComponent {
  error?: string;
  constructor(public activeModal: NgbActiveModal, private toastr: ToastrService) {
  }

  async copy() {
    await writeText(this.error!);
    this.toastr.success("Copied error");
  }
}
