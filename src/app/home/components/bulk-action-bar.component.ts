import { Component, EventEmitter, Input, Output } from '@angular/core';
import { CommonModule } from '@angular/common';
import { trigger, transition, style, animate } from '@angular/animations';

@Component({
  selector: 'app-bulk-action-bar',
  standalone: true,
  imports: [CommonModule],
  animations: [
    trigger('fadeInOut', [
      transition(':enter', [
        style({ opacity: 0, height: 0, padding: '0', margin: '0' }),
        animate('250ms', style({ opacity: 1, height: '*', padding: '*', margin: '*' })),
      ]),
      transition(':leave', [
        style({ opacity: 1, height: '*', padding: '*', margin: '*' }),
        animate('250ms', style({ opacity: 0, height: 0, padding: '0', margin: '0' })),
      ]),
    ]),
  ],
  template: `
    <div class="bulk-sticky-bar" *ngIf="visible" [@fadeInOut]>
      <div class="bulk-bar-content">
        <button class="bulk-btn btn-fav" (click)="action.emit('Favorite')">
          <svg viewBox="0 0 24 24">
            <path
              d="M12,17.27L18.18,21L16.54,13.97L22,9.24L14.81,8.62L12,2L9.19,8.62L2,9.24L7.45,13.97L5.82,21L12,17.27Z"
            />
          </svg>
          Favorite
        </button>
        <button class="bulk-btn btn-hide" (click)="action.emit('Hide')">
          <svg viewBox="0 0 24 24">
            <path
              d="M11.83,9L15,12.17V12.13C15,10.96 14.04,10 12.87,10H12.83L11.83,9M12,13.17L10.83,12H10.87C12.04,12 13,12.96 13,14.13V14.17L12,13.17M12,4.5C7,4.5 2.73,7.61 1,12C2.73,16.39 7,19.5 12,19.5C17,19.5 21.27,16.39 23,12C21.27,7.61 17,4.5 12,4.5M12,17A5,5 0 0,1 7,12A5,5 0 0,1 12,7A5,5 0 0,1 17,12A5,5 0 0,1 12,17M12,9A3,3 0 0,0 9,12A3,3 0 0,0 12,15A3,3 0 0,0 15,12A3,3 0 0,0 12,9Z"
            />
          </svg>
          Hide
        </button>
        <button class="bulk-btn btn-whitelist" (click)="action.emit('Whitelist')">
          <svg viewBox="0 0 24 24">
            <path
              d="M12,2A10,10 0 1,0 22,12A10,10 0 0,0 12,2M12,18A6,6 0 1,1 18,12A6,6 0 0,1 12,18M12,8A4,4 0 1,0 16,12A4,4 0 0,0 12,8Z"
            />
          </svg>
          Whitelist
        </button>
        <div class="spacer"></div>
        <button class="bulk-btn btn-cancel" (click)="cancel.emit()">Cancel</button>
      </div>
    </div>
  `,
  styleUrl: './bulk-action-bar.component.css',
})
export class BulkActionBarComponent {
  @Input() visible = false;
  @Output() action = new EventEmitter<string>();
  @Output() cancel = new EventEmitter<void>();
}
