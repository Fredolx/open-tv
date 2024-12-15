import { Component, Input } from '@angular/core';
import { EPG } from '../../models/epg';

@Component({
  selector: 'app-epg-modal-item',
  templateUrl: './epg-modal-item.component.html',
  styleUrl: './epg-modal-item.component.css'
})
export class EpgModalItemComponent {
  @Input()
  epg?: EPG
}
