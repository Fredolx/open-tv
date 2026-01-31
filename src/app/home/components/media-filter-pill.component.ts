import { Component, EventEmitter, Input, Output } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { MediaType } from '../../models/mediaType';

@Component({
  selector: 'app-media-filter-pill',
  standalone: true,
  imports: [CommonModule, FormsModule],
  template: `
    <div class="media-switch-pill">
      <label class="switch-item" [class.active]="chkLiveStream">
        <input
          type="checkbox"
          [(ngModel)]="chkLiveStream"
          (ngModelChange)="toggle.emit(mediaTypeEnum.livestream)"
        />
        <span class="dot live"></span>
        <span>Live TV</span>
      </label>
      <label class="switch-item" [class.active]="chkMovie">
        <input
          type="checkbox"
          [(ngModel)]="chkMovie"
          (ngModelChange)="toggle.emit(mediaTypeEnum.movie)"
        />
        <span class="dot movie"></span>
        <span>Movies</span>
      </label>
      <label class="switch-item" [class.active]="chkSerie" *ngIf="showSeries">
        <input
          type="checkbox"
          [(ngModel)]="chkSerie"
          (ngModelChange)="toggle.emit(mediaTypeEnum.serie)"
        />
        <span class="dot series"></span>
        <span>Series</span>
      </label>
    </div>
  `,
  styleUrl: './media-filter-pill.component.css',
})
export class MediaFilterPillComponent {
  @Input() chkLiveStream = true;
  @Input() chkMovie = true;
  @Input() chkSerie = true;
  @Input() showSeries = false;
  @Output() toggle = new EventEmitter<number>();

  mediaTypeEnum = MediaType;
}
