import { Component } from '@angular/core';
import { NgbActiveModal } from '@ng-bootstrap/ng-bootstrap';
import { EPG } from '../models/epg';

@Component({
  selector: 'app-epg-modal',
  templateUrl: './epg-modal.component.html',
  styleUrl: './epg-modal.component.css'
})
export class EpgModalComponent {
  name?: string;
  epg: EPG[] = [];
  constructor(public activeModal: NgbActiveModal) {

  }
}
