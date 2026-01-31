import { Component, EventEmitter, Input, Output } from '@angular/core';
import { CommonModule } from '@angular/common';
import { MediaType } from '../../models/mediaType';

export interface FilterChip {
  id: string;
  label: string;
  icon?: string;
  active: boolean;
  type: 'media' | 'quality' | 'misc';
  value?: any;
}

@Component({
  selector: 'app-filter-chips',
  standalone: true,
  imports: [CommonModule],
  template: `
    <div class="chips-container no-scrollbar">
      <div
        *ngFor="let chip of chips"
        class="chip"
        [class.active]="chip.active"
        (click)="toggleChip(chip)"
        [title]="chip.label"
      >
        <span class="chip-label">{{ chip.label }}</span>
      </div>
    </div>
  `,
  styles: [
    `
      .chips-container {
        display: flex;
        gap: 10px;
        overflow-x: auto;
        padding: 10px 0;
        scrollbar-width: none;
        -ms-overflow-style: none;
        mask-image: linear-gradient(to right, black 90%, transparent 100%);
      }
      .chips-container::-webkit-scrollbar {
        display: none;
      }
      .chip {
        padding: 8px 16px;
        border-radius: 20px;
        background: rgba(255, 255, 255, 0.05);
        color: rgba(255, 255, 255, 0.7);
        font-size: 0.85rem;
        font-weight: 500;
        cursor: pointer;
        transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
        border: 1px solid rgba(255, 255, 255, 0.1);
        white-space: nowrap;
        user-select: none;
      }
      .chip:hover {
        background: rgba(255, 255, 255, 0.1);
        color: #fff;
        transform: translateY(-1px);
        border-color: rgba(255, 255, 255, 0.25);
      }
      .chip.active {
        background: var(--beats-red);
        color: #fff;
        border-color: var(--beats-red);
        box-shadow: 0 4px 12px var(--beats-red-glow);
      }
    `,
  ],
})
export class FilterChipsComponent {
  @Input() chips: FilterChip[] = [];
  @Output() filterChanged = new EventEmitter<FilterChip>();

  toggleChip(chip: FilterChip) {
    this.filterChanged.emit(chip);
  }
}
