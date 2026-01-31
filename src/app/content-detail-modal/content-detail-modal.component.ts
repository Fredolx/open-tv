import { Component, EventEmitter, HostBinding, Input, Output } from '@angular/core';
import { CommonModule } from '@angular/common';
import { Channel } from '../models/channel';

@Component({
  selector: 'app-content-detail-modal',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './content-detail-modal.component.html',
  styleUrls: ['./content-detail-modal.component.css'],
})
export class ContentDetailModalComponent {
  @Input() channel: Channel | null = null;
  @Input()
  @HostBinding('class.open')
  isOpen = false;

  @Input() isLoadingDetails = false;

  @Output() close = new EventEmitter<void>();
  @Output() play = new EventEmitter<void>();
}
