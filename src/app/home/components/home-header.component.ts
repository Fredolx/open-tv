import { Component, EventEmitter, Input, Output } from '@angular/core';
import { CommonModule } from '@angular/common';
import { MatMenuModule } from '@angular/material/menu';

@Component({
  selector: 'app-home-header',
  standalone: true,
  imports: [CommonModule, MatMenuModule],
  template: `
    <header class="app-header">
      <div class="header-content">
        <div class="logo-section">
          <h1 class="app-title">Beats TV</h1>
          <span class="app-subtitle">IPTV Player</span>
        </div>

        <div class="selection-status" *ngIf="selectedCount > 0">
          <span class="selection-count">{{ selectedCount }} items selected</span>
          <button class="btn-clear" (click)="clearSelection.emit()">Clear</button>
        </div>

        <div class="header-actions">
          <button
            class="icon-btn"
            (click)="toggleSelectionMode.emit()"
            [class.active]="selectionMode"
            title="Toggle Selection Mode"
          >
            <svg viewBox="0 0 24 24" fill="currentColor">
              <path
                d="M3,5H9V11H3V5M5,7V9H7V7H5M11,7H21V9H11V7M11,11H21V13H11V11M5,13V15H7V13H5M3,11H9V17H3V11M11,15H21V17H11V15M5,19V21H7V19H5M3,17H9V23H3V17M11,19H21V21H11V19Z"
              />
            </svg>
          </button>

          <button
            class="icon-btn"
            [matMenuTriggerFor]="bulkMenu"
            [class.disabled]="bulkDisabled"
            title="Bulk Actions"
          >
            <svg viewBox="0 0 24 24" fill="currentColor">
              <path
                d="M6,2A4,4 0 0,1 10,6V8H14V6A4,4 0 0,1 18,2A4,4 0 0,1 22,6A4,4 0 0,1 18,10H16V14H18A4,4 0 0,1 22,18A4,4 0 0,1 18,22A4,4 0 0,1 14,18V16H10V18A4,4 0 0,1 6,22A4,4 0 0,1 2,18A4,4 0 0,1 6,14H8V10H6A4,4 0 0,1 2,6A4,4 0 0,1 6,2M16,18A2,2 0 0,0 18,20A2,2 0 0,0 20,18A2,2 0 0,0 18,16H16V18M14,10H10V14H14V10M6,16A2,2 0 0,0 4,18A2,2 0 0,0 6,20A2,2 0 0,0 8,18V16H6M8,6A2,2 0 0,0 6,4A2,2 0 0,0 4,6A2,2 0 0,0 6,8H8V6M18,8A2,2 0 0,0 20,6A2,2 0 0,0 18,4A2,2 0 0,0 16,6V8H18Z"
              />
            </svg>
          </button>

          <button class="settings-btn" (click)="openSettings.emit()" title="Settings">
            <svg fill="currentColor" viewBox="0 0 24 24">
              <path
                d="M12,15.5A3.5,3.5 0 0,1 8.5,12A3.5,3.5 0 0,1 12,8.5A3.5,3.5 0 0,1 15.5,12A3.5,3.5 0 0,1 12,15.5M19.43,12.97C19.47,12.65 19.5,12.33 19.5,12C19.5,11.67 19.47,11.34 19.43,11L21.54,9.37C21.73,9.22 21.78,8.95 21.66,8.73L19.66,5.27C19.54,5.05 19.27,4.96 19.05,5.05L16.56,6.05C16.04,5.66 15.5,5.32 14.87,5.07L14.5,2.42C14.46,2.18 14.25,2 14,2H10C9.75,2 9.54,2.18 9.5,2.42L9.13,5.07C8.5,5.32 7.96,5.66 7.44,6.05L4.95,5.05C4.73,4.96 4.46,5.05 4.34,5.27L2.34,8.73C2.21,8.95 2.27,9.22 2.46,9.37L4.57,11C4.53,11.34 4.5,11.67 4.5,12C4.5,12.33 4.53,12.65 4.57,12.97L2.46,14.63C2.27,14.78 2.21,15.05 2.34,15.27L4.34,18.73C4.46,18.95 4.73,19.03 4.95,18.95L7.44,17.94C7.96,18.34 8.5,18.68 9.13,18.93L9.5,21.58C9.54,21.82 9.75,22 10,22H14C14.25,22 14.46,21.82 14.5,21.58L14.87,18.93C15.5,18.67 16.04,18.34 16.56,17.94L19.05,18.95C19.27,19.03 19.54,18.95 19.66,18.73L21.66,15.27C21.78,15.05 21.73,14.78 21.54,14.63L19.43,12.97Z"
              />
            </svg>
          </button>
        </div>
      </div>
    </header>
  `,
  styleUrl: './home-header.component.css',
})
export class HomeHeaderComponent {
  @Input() selectedCount = 0;
  @Input() selectionMode = false;
  @Input() bulkDisabled = false;
  @Input() bulkMenu: any;

  @Output() toggleSelectionMode = new EventEmitter<void>();
  @Output() clearSelection = new EventEmitter<void>();
  @Output() openSettings = new EventEmitter<void>();
}
