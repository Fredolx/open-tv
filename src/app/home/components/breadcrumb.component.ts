import { Component, EventEmitter, Input, Output } from '@angular/core';
import { CommonModule } from '@angular/common';

@Component({
  selector: 'app-breadcrumb',
  standalone: true,
  imports: [CommonModule],
  template: `
    <div class="breadcrumb" *ngIf="visible">
      <button class="back-btn" (click)="back.emit()" title="Go Back">
        <svg viewBox="0 0 24 24" fill="currentColor">
          <path
            d="M2,12A10,10 0 0,1 12,2A10,10 0 0,1 22,12A10,10 0 0,1 12,22A10,10 0 0,1 2,12M18,11H10L13.5,7.5L12.08,6.08L6.16,12L12.08,17.92L13.5,16.5L10,13H18V11Z"
          />
        </svg>
      </button>
      <h4 class="breadcrumb-text">
        {{ text }}
      </h4>
    </div>
  `,
  styleUrl: './breadcrumb.component.css',
})
export class BreadcrumbComponent {
  @Input() visible = false;
  @Input() text = '';
  @Output() back = new EventEmitter<void>();
}
